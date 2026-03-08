# Checklist Pre-PR - ZeroClaw OpenAI-Compatible Transcription

## ⚠️ BLOCKERS (Da Risolvere Prima della PR)

### 1. Bug HTTP Timeout ❌ CRITICO
**Status**: In corso - TCP connection funziona ma HTTP request si blocca

**Problema**:
- ZeroClaw si connette correttamente a Whisper (TCP handshake ok)
- La richiesta HTTP POST multipart non arriva mai al server Whisper
- Telegram mostra il messaggio come "inviato ma non letto" (processing bloccato)
- Nessun errore nei log dopo "connected to 172.21.0.3:8000"

**Debugging in corso**:
- Log monitoring attivo (due terminali in background)
- In attesa di test con messaggio vocale breve (2-3 secondi)

**Possibili cause**:
- Timeout nella richiesta HTTP
- Problema con multipart form encoding
- Chunked transfer encoding non supportato dal server
- Connection pooling issue

**Azioni next**:
1. Inviare messaggio vocale breve e osservare log real-time
2. Verificare timeout configuration in `build_runtime_proxy_client()`
3. Testare con curl diretto: `curl -X POST -F file=@test.ogg http://localhost:8000/v1/audio/transcriptions`
4. Controllare se model="small" è valido per faster-whisper-server
5. Verificare Content-Type headers per multipart/form-data

**Impact**: BLOCCA LA PR - non possiamo inviare una feature che non funziona

---

## ✅ VALIDAZIONE TECNICA (Obbligatoria secondo CONTRIBUTING.md)

### 2. Rust Quality Gate
```bash
cd C:\Users\marco\Progetti\radosoft\zeroclaw-fork

# Formatting check
cargo fmt --all -- --check

# Lint check (strict)
cargo clippy --all-targets -- -D warnings

# Test suite (must pass)
cargo test --locked
```

**Status**: Non eseguito (Rust toolchain non configurato su Windows)  
**Soluzione**: 
- Eseguire in Docker CI environment
- O configurare Rust locale con `rustup default stable`
- GitHub Actions CI farà validation automatica

### 3. Test Coverage - Modifiche Necessarie

**File**: `src/channels/transcription.rs`

#### Test da AGGIORNARE:
```rust
#[tokio::test]
async fn rejects_missing_api_key() {
    // ❌ PROBLEMA: Questo test fallirà perché ora l'API key è opzionale
    // per provider senza auth (es. local Whisper)
    
    // Opzione A: Rimuovere il test (API key non più obbligatoria)
    // Opzione B: Modificare per testare solo provider Groq (che richiede key)
}
```

#### Test da AGGIUNGERE:
```rust
#[test]
fn api_key_resolution_prefers_config() {
    // Test che config.api_key ha precedenza su GROQ_API_KEY
}

#[test]
fn api_key_filters_placeholder_values() {
    // Test che "not-required", "none", "" vengono filtrati
}

#[test]
fn authorization_header_skipped_for_placeholders() {
    // Test che Authorization header non viene aggiunto per placeholder
}

#[tokio::test]
async fn works_with_openai_compatible_provider() {
    // Integration test con mock server OpenAI-compatible
}
```

---

## 📋 CONTRIBUTING.md Compliance Checklist

### Definition of Ready (DoR) - Prima di Aprire la PR

- [x] Scope è focalizzato su singola feature (transcription provider support)
- [x] Template PR completato (PR_DESCRIPTION.md)
- [ ] Validazione locale eseguita (`fmt`, `clippy`, `test`) - **PENDING**
- [x] Security impact descritto esplicitamente
- [x] Rollback path definito
- [x] No dati personali/sensibili in code/docs/tests/commits
- [x] Test usano wording neutro e project-scoped
- [x] Linked issue incluso (o rationale)

### Definition of Done (DoD) - Prima del Merge

- [ ] `CI Required Gate` verde su GitHub Actions
- [ ] Required reviewers approved (1 maintainer per Track B)
- [ ] Risk label corretto (`risk: medium`)
- [ ] User-visible behavior documentato
- [ ] Migration notes complete (già fatto - backward compatible)
- [ ] Follow-up TODOs tracked in issues

### Track Assignment

**Track B (Medium risk)** ✅ Corretto
- Motivo: Modifica comportamento channels/transcription
- Review requirement: 1 subsystem-aware review + explicit validation evidence
- NON Track A (low risk) perché tocca runtime behavior
- NON Track C (high risk) perché non tocca security/runtime/gateway/CI core

### PR Labels da Richiedere

```
risk: medium
size: S         # auto-managed
scope: channel
scope: config
module: channel: transcription
module: config: schema
```

### Commit Convention ✅ GIÀ RISPETTATO

```
✅ 6c2d1e8 feat(channels): add OpenAI-compatible transcription provider support
✅ 1fc2f42 fix(channels): skip Authorization header for auth-free transcription providers
```

Entrambi seguono Conventional Commits con scope appropriato.

### Code Quality Requirements

- [x] Naming conventions rispettate (snake_case, PascalCase, domain-first)
- [x] Architecture boundaries rispettate (no cross-subsystem coupling)
- [x] Trait-based pluggability mantenuta
- [x] Config schema trattato come public contract
- [x] Default values per backward compatibility
- [ ] Inline tests aggiunti/aggiornati - **PENDING**
- [x] No new dependencies (binary size optimization)
- [x] No `unwrap()` in production code

### Secret Hygiene ✅ VERIFICATO

```bash
# Verifica nessun secret committato
cd C:\Users\marco\Progetti\radosoft\zeroclaw-fork
git log -p | Select-String -Pattern "(api[_-]?key|secret|token|password|bearer|sk-)" -Context 2

# Verifica .env non staged
git status --short | Select-String "\.env$"
```

**Status**: ✅ Clean
- No API keys reali nei commit
- Solo placeholder values ("not-required", "none")
- Config examples usano endpoint locali

---

## 🚀 AZIONI DA COMPLETARE (In Ordine)

### Immediate (Prima di Aprire PR)

1. **[BLOCCO] Risolvere HTTP timeout bug**
   - Test con messaggio vocale breve
   - Debug HTTP request construction
   - Verificare multipart encoding
   - Testare curl diretto contro Whisper

2. **Aggiornare test `rejects_missing_api_key`**
   - Rimuovere o modificare per testare solo Groq
   - API key ora opzionale per provider auth-free

3. **Aggiungere nuovi test**
   - Config-first API key resolution
   - Placeholder filtering ("not-required", "none", "")
   - Authorization header skip logic

4. **Eseguire Rust quality gate**
   ```bash
   ./scripts/ci/rust_quality_gate.sh
   cargo test --locked
   ```

5. **Verificare no secrets**
   ```bash
   git log -p | grep -iE '(api[_-]?key|secret|token)'
   ```

### Post-PR (Se richiesto da reviewer)

6. **Squash commits** se maintainer lo richiede
   ```bash
   git rebase -i HEAD~2
   # Squash 1fc2f42 into 6c2d1e8
   ```

7. **Aggiornare documentazione** se richiesto
   - README.md con esempio configurazione Whisper locale
   - TESTING_TELEGRAM.md con test voice transcription

8. **Integration tests** se richiesto
   - Test con tutti i channel types
   - Test con diversi modelli Whisper (base, small, medium, large)
   - Performance test con file audio grandi

---

## 📊 Current Status Summary

| Item | Status | Priority | Blocking PR? |
|------|--------|----------|--------------|
| HTTP timeout bug | ❌ In corso | 🔴 CRITICAL | ✅ YES |
| Rust fmt/clippy/test | ⏳ Pending | 🟡 HIGH | ✅ YES |
| Test coverage update | ⏳ Pending | 🟡 HIGH | ⚠️ SOFT BLOCK |
| PR description | ✅ Done | 🟢 HIGH | ❌ NO |
| Commit convention | ✅ Done | 🟢 MEDIUM | ❌ NO |
| Secret hygiene | ✅ Clean | 🟢 MEDIUM | ❌ NO |
| Documentation | ⏳ Optional | 🔵 LOW | ❌ NO |

**VERDICT**: ⛔ **NON PRONTA PER PR**
- **Blocker principale**: HTTP timeout bug deve essere risolto
- **Blocker secondario**: Test suite deve passare
- **Soft blocker**: Test coverage da aggiornare

**Tempo stimato prima di PR**: 2-4 ore di debugging + testing

---

## 🎯 Next Steps Immediati

1. **Aspettare test voice message utente** (monitoring attivo)
2. **Analizzare log real-time** per capire punto di failure
3. **Fix HTTP timeout** con soluzione appropriata
4. **Aggiornare test** per riflettere nuova logica API key
5. **Eseguire quality gate** locale o in Docker
6. **Aprire PR su GitHub** contro branch `dev`

---

## 📝 Note per il Maintainer

**Punti di attenzione per review**:

1. **Backward compatibility**: Verificare che setup Groq esistenti funzionino unchanged
2. **Security**: API key handling, placeholder filtering, header skip
3. **Config schema**: snake_case fields, serde defaults corretti
4. **Test coverage**: Verificare adeguatezza test per nuova logica
5. **Error messages**: Chiari e actionable per troubleshooting

**Domande attese**:

- Q: "Perché non usare feature flag per rollout graduale?"
  - A: Backward compatible by design, nessun breaking change

- Q: "Performance impact del config lookup?"
  - A: Minimo - risolve una volta all'avvio, poi cached

- Q: "Supporto per altri provider (Azure, AWS Transcribe)?"
  - A: OpenAI-compatible interface - qualsiasi endpoint compatibile dovrebbe funzionare

**Follow-up issues suggeriti**:

1. [ ] Documentation: Add Whisper local setup guide
2. [ ] Feature: Support streaming transcription (incremental results)
3. [ ] Testing: Add integration tests for all channel types
4. [ ] Performance: Benchmark transcription latency by model size
