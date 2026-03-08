use anyhow::{bail, Context, Result};
use reqwest::multipart::{Form, Part};

use crate::config::TranscriptionConfig;

/// Maximum upload size accepted by the Groq Whisper API (25 MB).
const MAX_AUDIO_BYTES: usize = 25 * 1024 * 1024;

/// Map file extension to MIME type for Whisper-compatible transcription APIs.
fn mime_for_audio(extension: &str) -> Option<&'static str> {
    match extension.to_ascii_lowercase().as_str() {
        "flac" => Some("audio/flac"),
        "mp3" | "mpeg" | "mpga" => Some("audio/mpeg"),
        "mp4" | "m4a" => Some("audio/mp4"),
        "ogg" | "oga" => Some("audio/ogg"),
        "opus" => Some("audio/opus"),
        "wav" => Some("audio/wav"),
        "webm" => Some("audio/webm"),
        _ => None,
    }
}

/// Normalize audio filename for Whisper-compatible APIs.
///
/// Groq validates the filename extension — `.oga` (Opus-in-Ogg) is not in
/// its accepted list, so we rewrite it to `.ogg`.
fn normalize_audio_filename(file_name: &str) -> String {
    match file_name.rsplit_once('.') {
        Some((stem, ext)) if ext.eq_ignore_ascii_case("oga") => format!("{stem}.ogg"),
        _ => file_name.to_string(),
    }
}

/// Transcribe audio bytes via a Whisper-compatible transcription API.
///
/// Returns the transcribed text on success.  Requires `GROQ_API_KEY` in the
/// environment.  The caller is responsible for enforcing duration limits
/// *before* downloading the file; this function enforces the byte-size cap.
pub async fn transcribe_audio(
    audio_data: Vec<u8>,
    file_name: &str,
    config: &TranscriptionConfig,
) -> Result<String> {
    if audio_data.len() > MAX_AUDIO_BYTES {
        bail!(
            "Audio file too large ({} bytes, max {MAX_AUDIO_BYTES})",
            audio_data.len()
        );
    }

    let normalized_name = normalize_audio_filename(file_name);
    let extension = normalized_name
        .rsplit_once('.')
        .map(|(_, e)| e)
        .unwrap_or("");
    let mime = mime_for_audio(extension).ok_or_else(|| {
        anyhow::anyhow!(
            "Unsupported audio format '.{extension}' — accepted: flac, mp3, mp4, mpeg, mpga, m4a, ogg, opus, wav, webm"
        )
    })?;

    // Resolve API key: config → env fallback → None (for providers that don't need auth)
    let api_key = if let Some(ref key) = config.api_key {
        let trimmed = key.trim();
        // Skip placeholder values that indicate "no auth needed"
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("not-required") || trimmed.eq_ignore_ascii_case("none") {
            None
        } else {
            Some(trimmed.to_string())
        }
    } else {
        // Fallback to GROQ_API_KEY for backward compatibility (Groq always needs auth)
        std::env::var("GROQ_API_KEY").ok().map(|k| k.trim().to_string()).filter(|k| !k.is_empty())
    };

    let client = crate::config::build_runtime_proxy_client_with_timeouts(
        "transcription.groq",
        config.timeout_secs,
        config.connect_timeout_secs,
    );

    let file_part = Part::bytes(audio_data)
        .file_name(normalized_name)
        .mime_str(mime)?;

    let mut form = Form::new()
        .part("file", file_part)
        .text("model", config.model.clone())
        .text("response_format", "json");

    if let Some(ref lang) = config.language {
        form = form.text("language", lang.clone());
    }

    let mut request = client
        .post(&config.api_url)
        .multipart(form);

    // Only add Authorization header if API key is present
    if let Some(ref key) = api_key {
        request = request.bearer_auth(key);
    }

    let resp = request
        .send()
        .await
        .context("Failed to send transcription request")?;

    let status = resp.status();
    let body: serde_json::Value = resp
        .json()
        .await
        .context("Failed to parse transcription response")?;

    if !status.is_success() {
        let error_msg = body["error"]["message"].as_str().unwrap_or("unknown error");
        bail!("Transcription API error ({}): {}", status, error_msg);
    }

    let text = body["text"]
        .as_str()
        .context("Transcription response missing 'text' field")?
        .to_string();

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_oversized_audio() {
        let big = vec![0u8; MAX_AUDIO_BYTES + 1];
        let config = TranscriptionConfig::default();

        let err = transcribe_audio(big, "test.ogg", &config)
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("too large"),
            "expected size error, got: {err}"
        );
    }

    #[test]
    fn transcription_config_has_timeout_defaults() {
        let config = TranscriptionConfig::default();
        assert_eq!(config.timeout_secs, 120, "default HTTP request timeout should be 120 seconds");
        assert_eq!(config.connect_timeout_secs, 10, "default TCP connect timeout should be 10 seconds");
    }

    #[test]
    fn transcription_config_respects_custom_timeouts() {
        let mut config = TranscriptionConfig::default();
        config.timeout_secs = 60;
        config.connect_timeout_secs = 5;
        
        assert_eq!(config.timeout_secs, 60);
        assert_eq!(config.connect_timeout_secs, 5);
    }

    #[test]
    fn api_key_filters_placeholder_not_required() {
        let mut config = TranscriptionConfig::default();
        config.api_key = Some("not-required".to_string());
        
        // Placeholder "not-required" should be filtered to None
        let api_key = if let Some(ref key) = config.api_key {
            let trimmed = key.trim();
            if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("not-required") || trimmed.eq_ignore_ascii_case("none") {
                None
            } else {
                Some(trimmed.to_string())
            }
        } else {
            None
        };
        
        assert!(api_key.is_none(), "placeholder 'not-required' should be filtered to None");
    }

    #[test]
    fn api_key_filters_placeholder_none() {
        let mut config = TranscriptionConfig::default();
        config.api_key = Some("none".to_string());
        
        let api_key = if let Some(ref key) = config.api_key {
            let trimmed = key.trim();
            if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("not-required") || trimmed.eq_ignore_ascii_case("none") {
                None
            } else {
                Some(trimmed.to_string())
            }
        } else {
            None
        };
        
        assert!(api_key.is_none(), "placeholder 'none' should be filtered to None");
    }

    #[test]
    fn api_key_filters_empty_string() {
        let mut config = TranscriptionConfig::default();
        config.api_key = Some("".to_string());
        
        let api_key = if let Some(ref key) = config.api_key {
            let trimmed = key.trim();
            if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("not-required") || trimmed.eq_ignore_ascii_case("none") {
                None
            } else {
                Some(trimmed.to_string())
            }
        } else {
            None
        };
        
        assert!(api_key.is_none(), "empty string should be filtered to None");
    }

    #[test]
    fn api_key_preserves_real_key() {
        let mut config = TranscriptionConfig::default();
        config.api_key = Some("sk-real-api-key-12345".to_string());
        
        let api_key = if let Some(ref key) = config.api_key {
            let trimmed = key.trim();
            if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("not-required") || trimmed.eq_ignore_ascii_case("none") {
                None
            } else {
                Some(trimmed.to_string())
            }
        } else {
            None
        };
        
        assert_eq!(api_key, Some("sk-real-api-key-12345".to_string()), "real API key should be preserved");
    }

    #[test]
    fn api_key_case_insensitive_placeholders() {
        let cases = [
            "NOT-REQUIRED",
            "Not-Required",
            "NONE",
            "None",
            "NoNe",
        ];
        
        for placeholder in cases {
            let mut config = TranscriptionConfig::default();
            config.api_key = Some(placeholder.to_string());
            
            let api_key = if let Some(ref key) = config.api_key {
                let trimmed = key.trim();
                if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("not-required") || trimmed.eq_ignore_ascii_case("none") {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            } else {
                None
            };
            
            assert!(api_key.is_none(), "placeholder '{}' should be filtered (case-insensitive)", placeholder);
        }
    }

    #[test]
    fn mime_for_audio_maps_accepted_formats() {
        let cases = [
            ("flac", "audio/flac"),
            ("mp3", "audio/mpeg"),
            ("mpeg", "audio/mpeg"),
            ("mpga", "audio/mpeg"),
            ("mp4", "audio/mp4"),
            ("m4a", "audio/mp4"),
            ("ogg", "audio/ogg"),
            ("oga", "audio/ogg"),
            ("opus", "audio/opus"),
            ("wav", "audio/wav"),
            ("webm", "audio/webm"),
        ];
        for (ext, expected) in cases {
            assert_eq!(
                mime_for_audio(ext),
                Some(expected),
                "failed for extension: {ext}"
            );
        }
    }

    #[test]
    fn mime_for_audio_case_insensitive() {
        assert_eq!(mime_for_audio("OGG"), Some("audio/ogg"));
        assert_eq!(mime_for_audio("MP3"), Some("audio/mpeg"));
        assert_eq!(mime_for_audio("Opus"), Some("audio/opus"));
    }

    #[test]
    fn mime_for_audio_rejects_unknown() {
        assert_eq!(mime_for_audio("txt"), None);
        assert_eq!(mime_for_audio("pdf"), None);
        assert_eq!(mime_for_audio("aac"), None);
        assert_eq!(mime_for_audio(""), None);
    }

    #[test]
    fn normalize_audio_filename_rewrites_oga() {
        assert_eq!(normalize_audio_filename("voice.oga"), "voice.ogg");
        assert_eq!(normalize_audio_filename("file.OGA"), "file.ogg");
    }

    #[test]
    fn normalize_audio_filename_preserves_accepted() {
        assert_eq!(normalize_audio_filename("voice.ogg"), "voice.ogg");
        assert_eq!(normalize_audio_filename("track.mp3"), "track.mp3");
        assert_eq!(normalize_audio_filename("clip.opus"), "clip.opus");
    }

    #[test]
    fn normalize_audio_filename_no_extension() {
        assert_eq!(normalize_audio_filename("voice"), "voice");
    }

    #[tokio::test]
    async fn rejects_unsupported_audio_format() {
        let data = vec![0u8; 100];
        let config = TranscriptionConfig::default();

        let err = transcribe_audio(data, "recording.aac", &config)
            .await
            .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("Unsupported audio format"),
            "expected unsupported-format error, got: {msg}"
        );
        assert!(
            msg.contains(".aac"),
            "error should mention the rejected extension, got: {msg}"
        );
    }
}
