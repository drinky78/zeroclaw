# 📦 Pull Request Materials - ZeroClaw OpenAI-Compatible Transcription

## 🎯 Obiettivo
Aggiungere supporto per provider di trascrizione OpenAI-compatible (es. Whisper locale) invece di essere limitati solo a Groq API.

---

## 📂 File Preparati per la PR

### 1. **PR_DESCRIPTION.md** ✅
**Scopo**: Testo completo della Pull Request da copiare su GitHub  
**Contenuto**:
- Summary (problema, soluzione, scope)
- Label snapshot richiesti
- Security impact e rollback plan
- Validation evidence
- Compatibility notes
- Human verification checklist

**Compliance**: Segue esattamente il template `.github/pull_request_template.md` del repository upstream

---

### 2. **PR_CHECKLIST.md** ✅
**Scopo**: Checklist dettagliata di tutti i requisiti CONTRIBUTING.md  
**Contenuto**:
- ⚠️ BLOCKERS da risolvere (HTTP timeout bug)
- ✅ Validazione tecnica (fmt, clippy, test)
- 📋 CONTRIBUTING.md compliance checklist
- Definition of Ready (DoR)
- Definition of Done (DoD)
- Track assignment (Track B - medium risk)
- Commit convention verification
- Secret hygiene checks

**Status Attuale**: 
- ❌ HTTP timeout bug (CRITICO - blocca PR)
- ⏳ Rust quality gate (da eseguire)
- ✅ Test coverage aggiornato (placeholder filtering tests)
- ✅ PR description completa
- ✅ Commit convention rispettata

---

### 3. **COME_APRIRE_PR.md** ✅
**Scopo**: Guida passo-passo per aprire la PR su GitHub  
**Contenuto**:
- ⚠️ Prerequisiti da verificare
- 📝 Preparazione finale del branch
- 🌐 Come aprire PR (web + CLI)
- 🏷️ Label e metadata da richiedere
- 👀 Post-submission checklist
- 🤖 Interazione con CI e reviewers
- 📊 Monitoring e follow-up
- ✅ Post-merge actions
- 🚨 Cosa NON fare

**Target**: `zeroclaw-labs/zeroclaw` branch `dev` (NON `main`)

---

### 4. **Modifiche al Codice** ✅

#### **src/config/schema.rs** (Commit 6c2d1e8)
```rust
// Aggiunti campi a TranscriptionConfig:
pub provider: String,           // default: "groq"
pub api_key: Option<String>,    // default: None

// Aggiunta funzione:
fn default_transcription_provider() -> String { "groq".into() }
```

**Backward compatibility**: ✅ Default values preservano comportamento esistente

---

#### **src/channels/transcription.rs** (Commit 6c2d1e8 + 1fc2f42)

**API Key Resolution Logic**:
```rust
let api_key = if let Some(ref key) = config.api_key {
    let trimmed = key.trim();
    // Skip placeholder values
    if trimmed.is_empty() || 
       trimmed.eq_ignore_ascii_case("not-required") || 
       trimmed.eq_ignore_ascii_case("none") {
        None
    } else {
        Some(trimmed.to_string())
    }
} else {
    // Fallback to GROQ_API_KEY for backward compatibility
    std::env::var("GROQ_API_KEY").ok()
        .map(|k| k.trim().to_string())
        .filter(|k| !k.is_empty())
};
```

**Authorization Header Skip**:
```rust
let mut request = client.post(&config.api_url).multipart(form);

// Only add Authorization header if API key is present
if let Some(ref key) = api_key {
    request = request.bearer_auth(key);
}
```

**Why this matters**: faster-whisper-server e altri provider locali rifiutano richieste con Authorization header invalidi

---

#### **Test Coverage** ✅ AGGIORNATO
```rust
// RIMOSSO test problematico:
// async fn rejects_missing_api_key() { ... }  ❌ API key non più obbligatoria

// AGGIUNTI nuovi test:
fn api_key_filters_placeholder_not_required()     // ✅
fn api_key_filters_placeholder_none()             // ✅
fn api_key_filters_empty_string()                 // ✅
fn api_key_preserves_real_key()                   // ✅
fn api_key_case_insensitive_placeholders()        // ✅
```

**Coverage**: Logica placeholder filtering, config-first resolution, case-insensitive matching

**Test esistenti preservati**:
- ✅ `rejects_oversized_audio`
- ✅ `mime_for_audio_maps_accepted_formats`
- ✅ `mime_for_audio_case_insensitive`
- ✅ `mime_for_audio_rejects_unknown`
- ✅ `normalize_audio_filename_rewrites_oga`
- ✅ `normalize_audio_filename_preserves_accepted`
- ✅ `normalize_audio_filename_no_extension`
- ✅ `rejects_unsupported_audio_format`

---

## 🔍 Commit History

```
1fc2f42 - fix(channels): skip Authorization header for auth-free transcription providers
6c2d1e8 - feat(channels): add OpenAI-compatible transcription provider support
```

**Conventional Commits**: ✅ Rispettati  
**Scope**: `(channels)` appropriato  
**Message**: Descrittivi e actionable

---

## 📊 Compliance Matrix

| Requirement | Status | Notes |
|-------------|--------|-------|
| **Conventional Commits** | ✅ Pass | `feat(channels):`, `fix(channels):` |
| **Code Formatting** | ⏳ Pending | Rust toolchain non configurato locale |
| **Lint Check** | ⏳ Pending | `cargo clippy` da eseguire |
| **Test Suite** | ⏳ Pending | `cargo test --locked` da eseguire |
| **Test Coverage** | ✅ Pass | Nuovi test per placeholder filtering |
| **Backward Compatibility** | ✅ Pass | Default values + env fallback |
| **Security Impact** | ✅ Documented | API key handling, placeholder filtering |
| **Rollback Plan** | ✅ Documented | Config revert + env var fallback |
| **Secret Hygiene** | ✅ Pass | No credentials committati |
| **PR Template** | ✅ Complete | PR_DESCRIPTION.md segue template |
| **Track Assignment** | ✅ Correct | Track B (medium risk) |
| **Target Branch** | ✅ Correct | `dev` (non `main`) |

---

## ⚠️ Blockers Rimanenti

### 1. **HTTP Timeout Bug** 🔴 CRITICO
**Status**: In debugging  
**Sintomo**: TCP connects to Whisper, HTTP POST doesn't complete  
**Impact**: BLOCCA APERTURA PR (feature non funziona)  
**Next Steps**:
1. Test con messaggio vocale breve
2. Analisi log real-time
3. Debug multipart request construction
4. Verifica timeout config

### 2. **Rust Quality Gate** 🟡 HIGH
**Status**: Non eseguito (toolchain issue)  
**Commands**:
```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
```
**Workaround**: Eseguire in Docker CI o configurare Rust locale

---

## 📝 Configuration Examples

### Local Whisper (faster-whisper-server)
```toml
[transcription]
enabled = true
provider = "openai-compatible"
api_url = "http://localhost:8000/v1/audio/transcriptions"
api_key = "not-required"  # skip auth header
model = "small"
language = "it"
max_duration_secs = 120
```

### Groq (Backward Compatible)
```toml
[transcription]
enabled = true
# provider defaults to "groq"
# api_url defaults to "https://api.groq.com/openai/v1/audio/transcriptions"
# api_key falls back to GROQ_API_KEY env var
model = "whisper-large-v3"
```

### OpenAI Whisper API
```toml
[transcription]
enabled = true
provider = "openai-compatible"
api_url = "https://api.openai.com/v1/audio/transcriptions"
api_key = "sk-your-openai-key"
model = "whisper-1"
```

---

## 🚀 Next Actions (Prioritized)

1. **[BLOCCO]** Risolvere HTTP timeout bug → End-to-end transcription test
2. **Eseguire `cargo fmt && cargo clippy && cargo test`** → CI validation
3. **Committare test updates** → `git add src/channels/transcription.rs`
4. **Aprire PR su GitHub** → Target `dev` branch
5. **Richiedere label** → `risk: medium`, `scope: channel`, `module: channel: transcription`
6. **Monitor CI** → GitHub Actions validation
7. **Rispondere a review feedback** → Iterazione con maintainer

---

## 📧 GitHub PR Info

**Repository**: `zeroclaw-labs/zeroclaw`  
**Base Branch**: `dev` ⚠️ NON `main`  
**Head Branch**: `drinky78:feat/openai-compatible-transcription`  
**Title**: `feat(channels): add OpenAI-compatible transcription provider support`  
**Labels**: `risk: medium`, `scope: channel`, `scope: config`, `module: channel: transcription`, `module: config: schema`  
**Review Track**: Track B (1 subsystem-aware review + validation evidence)

---

## 🎓 Lessons Learned

1. **CONTRIBUTING.md è dettagliatissimo** → Leggere attentamente prima di aprire PR
2. **Track system** → Risk-based review depth (A/B/C)
3. **Conventional Commits obbligatori** → `feat(scope):`, `fix(scope):`
4. **Target `dev` non `main`** → Main è protected, dev è development branch
5. **Backward compatibility critico** → Default values + env fallback
6. **Test coverage importante** → Update test quando logica cambia
7. **Secret hygiene** → No credentials, use placeholders in examples
8. **HTTP timeout debugging è complesso** → TCP ≠ HTTP completion

---

## 📚 Reference Documents

- **Upstream CONTRIBUTING.md**: [Link](https://github.com/zeroclaw-labs/zeroclaw/blob/main/CONTRIBUTING.md)
- **PR Template**: [Link](https://github.com/zeroclaw-labs/zeroclaw/blob/main/.github/pull_request_template.md)
- **Fork Repository**: https://github.com/drinky78/zeroclaw
- **Feature Branch**: `feat/openai-compatible-transcription`
- **Working Project**: `C:\Users\marco\Progetti\radosoft\zeroclaw\`
- **Fork Clone**: `C:\Users\marco\Progetti\radosoft\zeroclaw-fork\`

---

## ✅ Ready for PR When:

- [ ] HTTP timeout bug resolved (transcription works end-to-end)
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo test --locked` all green
- [ ] Test updates committed and pushed
- [ ] PR_DESCRIPTION.md reviewed and finalized
- [ ] No secrets in git history verified

**Estimated time to PR-ready**: 2-4 hours (debugging + validation)

---

**Preparato da**: GitHub Copilot + User @drinky78  
**Data**: 2026-03-08  
**Status**: 🟡 MATERIALS READY, WAITING FOR BUG FIX
