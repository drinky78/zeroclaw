# HTTP Timeout Bug - Analisi e Soluzioni

## 🔍 Root Cause Identificata

**Problema**: `src/channels/transcription.rs` linea 76 usa:
```rust
let client = crate::config::build_runtime_proxy_client("transcription.groq");
```

**Issue**: `build_runtime_proxy_client()` NON imposta timeout sul client HTTP.

**Esiste alternativa**: `build_runtime_proxy_client_with_timeouts(service_key, timeout_secs, connect_timeout_secs)`

**Comportamento attuale**:
- ✅ TCP connect funziona (handshake completo)
- ❌ HTTP POST multipart si blocca all'infinito (nessun request timeout)
- ❌ Nessun errore nei log (semplicemente appeso)

---

## 📋 Domande per Valutare le Soluzioni

### 1. Correttezza Tecnica
- **Q1.1**: La soluzione risolve il timeout dell'HTTP request?
- **Q1.2**: La soluzione mantiene il TCP connect timeout appropriato?
- **Q1.3**: La soluzione gestisce correttamente timeout diversi per connect vs request?

### 2. Configurabilità
- **Q2.1**: Gli utenti possono configurare i timeout tramite config.toml?
- **Q2.2**: Esistono default ragionevoli se l'utente non configura?
- **Q2.3**: La configurazione segue i pattern esistenti in ZeroClaw?

### 3. Backward Compatibility
- **Q3.1**: Setup Groq esistenti continuano a funzionare senza modifiche?
- **Q3.2**: Nessun breaking change nella config schema?
- **Q3.3**: Default values preservano comportamento esistente (dove sensato)?

### 4. User Experience
- **Q4.1**: Messaggi di errore chiari quando timeout scade?
- **Q4.2**: Timeout ragionevoli per file audio (potenzialmente grandi)?
- **Q4.3**: Feedback tempestivo su failure (non 10 minuti di attesa)?

### 5. Maintainability
- **Q5.1**: Codice segue pattern esistenti in transcription.rs?
- **Q5.2**: Documentazione inline adeguata?
- **Q5.3**: Nessuna duplicazione di logica timeout?

### 6. Error Handling
- **Q6.1**: Distingue chiaramente connect timeout vs request timeout negli errori?
- **Q6.2**: Log sufficienti per debugging?
- **Q6.3**: Errore propagato correttamente all'utente Telegram?

### 7. Performance
- **Q7.1**: Timeout non troppo corti (false positive su audio grandi)?
- **Q7.2**: Timeout non troppo lunghi (UX pessima su failure)?
- **Q7.3**: Client HTTP cached appropriatamente?

### 8. Testing
- **Q8.1**: Come verificare che timeout funzioni?
- **Q8.2**: Come testare senza aspettare timeout reale?
- **Q8.3**: Test coverage adeguato per timeout logic?

---

## 💡 Soluzione 1: Timeout Fissi Hardcoded

### Implementazione
```rust
// In transcription.rs linea 76:
let client = crate::config::build_runtime_proxy_client_with_timeouts(
    "transcription.groq", 
    120,  // 2 minuti request timeout (per audio grandi)
    10    // 10 secondi connect timeout
);
```

### Valutazione

| Categoria | Score | Dettagli |
|-----------|-------|----------|
| **Correttezza Tecnica** | ✅ 9/10 | Risolve timeout, valori ragionevoli |
| **Configurabilità** | ❌ 2/10 | Nessuna configurazione utente |
| **Backward Compatibility** | ✅ 10/10 | Zero breaking changes |
| **User Experience** | ⚠️ 6/10 | Timeout fisso non adattabile |
| **Maintainability** | ✅ 8/10 | Semplice, pattern esistente |
| **Error Handling** | ✅ 7/10 | Usa error handling esistente |
| **Performance** | ⚠️ 7/10 | 120s può essere troppo per piccoli file |
| **Testing** | ✅ 8/10 | Facile testare con mock |

**TOTAL: 57/80 (71%)**

### Pro
- ✅ Implementazione immediate (1 linea modificata)
- ✅ Zero breaking changes
- ✅ Timeout ragionevoli per la maggior parte dei casi

### Contro
- ❌ Non configurabile dall'utente
- ❌ Valori hardcoded non seguono pattern ZeroClaw
- ❌ Difficile ottimizzare senza rebuild

---

## 💡 Soluzione 2: Timeout Configurabili in TranscriptionConfig

### Implementazione

**Step 1**: Aggiungere campi a `TranscriptionConfig` in `schema.rs`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TranscriptionConfig {
    // ... campi esistenti ...
    
    /// HTTP request timeout in seconds (default: 120)
    #[serde(default = "default_transcription_timeout_secs")]
    pub timeout_secs: u64,
    
    /// TCP connect timeout in seconds (default: 10)
    #[serde(default = "default_transcription_connect_timeout_secs")]
    pub connect_timeout_secs: u64,
}

fn default_transcription_timeout_secs() -> u64 {
    120  // 2 minuti per audio grandi
}

fn default_transcription_connect_timeout_secs() -> u64 {
    10  // 10 secondi per TCP handshake
}
```

**Step 2**: Aggiornare `Default` impl:
```rust
impl Default for TranscriptionConfig {
    fn default() -> Self {
        Self {
            // ... campi esistenti ...
            timeout_secs: default_transcription_timeout_secs(),
            connect_timeout_secs: default_transcription_connect_timeout_secs(),
        }
    }
}
```

**Step 3**: Usare in `transcription.rs`:
```rust
let client = crate::config::build_runtime_proxy_client_with_timeouts(
    "transcription.groq",
    config.timeout_secs,
    config.connect_timeout_secs,
);
```

### Valutazione

| Categoria | Score | Dettagli |
|-----------|-------|----------|
| **Correttezza Tecnica** | ✅ 10/10 | Completa, flessibile |
| **Configurabilità** | ✅ 10/10 | Completamente configurabile |
| **Backward Compatibility** | ✅ 10/10 | Default values = zero breaking |
| **User Experience** | ✅ 9/10 | Ottimizzabile per caso d'uso |
| **Maintainability** | ✅ 9/10 | Segue pattern ZeroClaw (HttpRequestConfig, WebSearchConfig) |
| **Error Handling** | ✅ 8/10 | Usa error handling esistente reqwest |
| **Performance** | ✅ 9/10 | Utente controlla trade-off |
| **Testing** | ✅ 8/10 | Testabile con config mock |

**TOTAL: 73/80 (91%)**

### Pro
- ✅ Configurabile tramite config.toml
- ✅ Default ragionevoli (backward compatible)
- ✅ Segue pattern esistente (timeout_secs in altre config)
- ✅ Utente può ottimizzare per caso d'uso
- ✅ Self-documenting (doc comments)

### Contro
- ⚠️ Richiede modifiche a schema.rs (più invasivo)
- ⚠️ Aumenta superficie config (più campi)

---

## 💡 Soluzione 3: Timeout Configurabili + Stima Dinamica

### Implementazione

**Step 1-2**: Come Soluzione 2 (campi config)

**Step 3**: Timeout dinamico basato su durata audio in `transcription.rs`:
```rust
// Calcola timeout dinamico basato su file size
// Stima: Whisper trascrive ~1MB audio in ~5 secondi (dipende dal modello)
let estimated_processing_secs = (audio_data.len() / (1024 * 1024)) as u64 * 5;
let dynamic_timeout = config.timeout_secs.max(estimated_processing_secs + 30);

let client = crate::config::build_runtime_proxy_client_with_timeouts(
    "transcription.groq",
    dynamic_timeout,
    config.connect_timeout_secs,
);

tracing::debug!(
    audio_size_bytes = audio_data.len(),
    estimated_processing_secs,
    timeout_secs = dynamic_timeout,
    "Transcription request with dynamic timeout"
);
```

### Valutazione

| Categoria | Score | Dettagli |
|-----------|-------|----------|
| **Correttezza Tecnica** | ⚠️ 7/10 | Stima può essere inaccurata |
| **Configurabilità** | ✅ 10/10 | Config + logica adattiva |
| **Backward Compatibility** | ✅ 10/10 | Default values preservano comportamento |
| **User Experience** | ✅ 10/10 | Ottimo: adatta automaticamente |
| **Maintainability** | ⚠️ 6/10 | Logica complessa, stima hardcoded |
| **Error Handling** | ✅ 8/10 | Buono + log dettagliati |
| **Performance** | ⚠️ 7/10 | Stima può essere troppo conservativa |
| **Testing** | ⚠️ 6/10 | Difficile testare stime accurate |

**TOTAL: 64/80 (80%)**

### Pro
- ✅ UX eccellente (adatta automaticamente)
- ✅ Log dettagliati per debugging
- ✅ Configurabile + intelligente

### Contro
- ❌ Stima processing time è unreliable (dipende da: modello Whisper, CPU, batch size, lingua)
- ❌ Complessità eccessiva per problema semplice
- ❌ Manutenzione più difficile
- ❌ Test coverage più complesso

---

## 📊 Confronto Finale

| Criterio | Soluzione 1 | Soluzione 2 | Soluzione 3 |
|----------|-------------|-------------|-------------|
| **Total Score** | 57/80 (71%) | **73/80 (91%)** | 64/80 (80%) |
| **Effort** | Minimo (1 LOC) | Medio (3 file) | Alto (3 file + logica) |
| **Risk** | Basso | Basso | Medio (stime inaccurate) |
| **CONTRIBUTING.md Compliance** | ✅ | ✅ | ⚠️ (over-engineering) |

---

## 🎯 Raccomandazione: **SOLUZIONE 2**

### Motivazione
1. **Score più alto** (91% vs 71% vs 80%)
2. **Segue pattern esistenti** in ZeroClaw (HttpRequestConfig, WebSearchConfig hanno timeout_secs)
3. **Configurabile** ma con default ragionevoli
4. **Backward compatible** (zero breaking changes)
5. **Maintainability** alta (no logica complessa)
6. **CONTRIBUTING.md compliant** (no over-engineering, trait-first thinking)

### Configurazione Utente Post-Fix
```toml
[transcription]
enabled = true
provider = "openai-compatible"
api_url = "http://whisper-stt:8000/v1/audio/transcriptions"
api_key = "not-required"
model = "small"
language = "it"
max_duration_secs = 120
timeout_secs = 120        # NEW: HTTP request timeout
connect_timeout_secs = 10 # NEW: TCP connect timeout
```

### Testing Plan
```bash
# 1. Test con timeout breve (deve failare)
# Modifica config: timeout_secs = 1
# Invia audio → deve vedere "Failed to send transcription request" entro 1 secondo

# 2. Test con timeout normale (deve funzionare)
# Modifica config: timeout_secs = 120
# Invia audio → deve transcrivere con successo

# 3. Test backward compat (deve usare defaults)
# Rimuovi timeout_secs dal config
# Invia audio → deve usare 120s default
```

---

## 🚀 Implementazione Scelta: Soluzione 2

**File da modificare**:
1. ✅ `src/config/schema.rs` - Aggiungere campi timeout a TranscriptionConfig
2. ✅ `src/channels/transcription.rs` - Usare build_runtime_proxy_client_with_timeouts
3. ✅ Aggiornare test in transcription.rs se necessario

**Commit message**:
```
feat(channels): add configurable HTTP timeouts for transcription requests

- Add timeout_secs (default: 120) to TranscriptionConfig
- Add connect_timeout_secs (default: 10) to TranscriptionConfig
- Use build_runtime_proxy_client_with_timeouts instead of build_runtime_proxy_client
- Fixes HTTP timeout bug where requests would hang indefinitely

Resolves: HTTP timeout issue with faster-whisper-server
```

**PR Impact**:
- ✅ Risolve blocker critico della PR
- ✅ Nessun breaking change
- ✅ Configurabile ma opzionale
- ✅ Segue CONTRIBUTING.md guidelines
