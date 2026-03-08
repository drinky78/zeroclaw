# Guida: Come Aprire la Pull Request su GitHub

## ⚠️ PREREQUISITI (DA VERIFICARE PRIMA)

Prima di procedere, verifica che:

- [ ] ✅ HTTP timeout bug risolto (transcription funziona end-to-end)
- [ ] ✅ `cargo fmt --all -- --check` passa
- [ ] ✅ `cargo clippy --all-targets -- -D warnings` passa  
- [ ] ✅ `cargo test --locked` passa (tutti i test green)
- [ ] ✅ Test `rejects_missing_api_key` aggiornato o rimosso
- [ ] ✅ Nuovi test aggiunti per config-first API key e header skip
- [ ] ✅ Nessun secret committato (`git log -p | grep -i secret`)

Se anche solo uno di questi ❌ NON è completo, **NON aprire la PR** - sarebbe rifiutata.

---

## 📝 Passo 1: Preparazione Finale del Branch

```bash
cd C:\Users\marco\Progetti\radosoft\zeroclaw-fork

# Verifica branch e commit
git status
git log --oneline -5

# Se necessario, squash commits (solo se richiesto da maintainer)
# git rebase -i HEAD~2
# Cambia "pick" in "squash" per 1fc2f42

# Push finale (se hai fatto squash o nuovi commit)
git push origin feat/openai-compatible-transcription -f  # -f solo se hai fatto rebase
```

---

## 🌐 Passo 2: Aprire la PR su GitHub

### Opzione A: Via Web Browser (Consigliato)

1. **Vai al tuo fork su GitHub**:
   ```
   https://github.com/drinky78/zeroclaw
   ```

2. **Clicca su "Compare & pull request"** (dovrebbe apparire automaticamente dopo il push)

3. **IMPORTANTE: Cambia il base branch a `dev`**
   ```
   Base repository: zeroclaw-labs/zeroclaw
   Base branch: dev  ← ASSICURATI CHE SIA "dev" NON "main"
   
   Head repository: drinky78/zeroclaw
   Compare branch: feat/openai-compatible-transcription
   ```

4. **Copia il contenuto di `PR_DESCRIPTION.md` nel form della PR**
   - Titolo: "feat(channels): add OpenAI-compatible transcription provider support"
   - Description: Copia tutto da PR_DESCRIPTION.md

5. **Aggiungi i label manualmente** (se hai i permessi) o **richiedi nei commenti**:
   ```
   risk: medium
   scope: channel
   scope: config
   module: channel: transcription
   module: config: schema
   ```

6. **Clicca "Create pull request"**

### Opzione B: Via GitHub CLI (se installato)

```bash
cd C:\Users\marco\Progetti\radosoft\zeroclaw-fork

# Installa GitHub CLI se non presente
# winget install --id GitHub.cli

# Login (prima volta)
gh auth login

# Crea la PR contro dev branch
gh pr create \
  --repo zeroclaw-labs/zeroclaw \
  --base dev \
  --head drinky78:feat/openai-compatible-transcription \
  --title "feat(channels): add OpenAI-compatible transcription provider support" \
  --body-file PR_DESCRIPTION.md \
  --label "risk: medium,scope: channel,scope: config"
```

---

## 🏷️ Passo 3: Label e Metadata (Post-Creation)

Dopo aver creato la PR, verifica/richiedi questi label:

### Label Obbligatori
- ✅ `risk: medium` - Modifica comportamento channel/transcription
- ✅ `scope: channel` - Scope primario
- ✅ `scope: config` - Scope secondario
- ✅ `module: channel: transcription` - Module specifico
- ✅ `module: config: schema` - Config touched

### Label Auto-Managed (Non Toccare)
- `size: S` - Calcolato automaticamente da bot
- `trusted contributor` / `experienced contributor` / etc. - Basato su PR count

Se non hai permessi per aggiungere label, aggiungi un commento:

```markdown
@maintainers Please add labels:
- risk: medium
- scope: channel, config
- module: channel: transcription, config: schema
```

---

## 👀 Passo 4: Post-Submission Checklist

Dopo aver aperto la PR, verifica che:

- [ ] ✅ Base branch è `dev` (NON `main`)
- [ ] ✅ PR description è completa e segue il template
- [ ] ✅ CI checks partono automaticamente (GitHub Actions)
- [ ] ✅ `CI Required Gate` workflow è verde
- [ ] ✅ Nessun merge conflict con `dev`
- [ ] ✅ Label appropriate aggiunte o richieste
- [ ] ✅ Linked issue presente (o spiegato perché non c'è)

---

## 🤖 Passo 5: Interazione con CI e Reviewers

### Quando CI Fallisce ❌

Se `CI Required Gate` è rosso:

1. **Controlla il workflow log** su GitHub Actions
2. **Errori comuni**:
   - `cargo fmt` failed → Esegui `cargo fmt --all` e commit fix
   - `cargo clippy` warnings → Risolvi warning e commit fix
   - `cargo test` failed → Debug test failure locale, fix e commit
   - Merge conflict → Resync con `dev` branch

3. **Fix e push**:
   ```bash
   # Fix locale
   cargo fmt --all
   cargo clippy --fix --allow-dirty
   
   # Commit fix
   git add .
   git commit -m "chore: fix CI linting issues"
   git push origin feat/openai-compatible-transcription
   ```

### Quando Reviewer Richiede Modifiche 📝

**Workflow per modifiche**:

```bash
cd C:\Users\marco\Progetti\radosoft\zeroclaw-fork

# Fai le modifiche richieste
# ... edit files ...

# Commit con messaggio descrittivo
git add .
git commit -m "fix: address review feedback - <descrizione specifica>"

# Push (aggiorna automaticamente la PR)
git push origin feat/openai-compatible-transcription
```

**NON:**
- ❌ Force push (`-f`) a meno che esplicitamente richiesto
- ❌ Chiudere e riaprire nuova PR
- ❌ Creare commit "wip" o "temp" (squash se necessario)

**SÌ:**
- ✅ Commit descrittivi per ogni feedback point
- ✅ Rispondere ai commenti con "Fixed in <commit_hash>"
- ✅ Re-request review dopo ogni round di fix
- ✅ Segnalare quando tutti i feedback sono risolti

### Squash Commits (Se Richiesto)

Se maintainer chiede di squash commits:

```bash
cd C:\Users\marco\Progetti\radosoft\zeroclaw-fork

# Interactive rebase per squash
git rebase -i HEAD~2  # se hai 2 commit da squash

# Nell'editor, cambia:
# pick 6c2d1e8 feat(channels): add OpenAI-compatible transcription provider support
# squash 1fc2f42 fix(channels): skip Authorization header for auth-free transcription providers

# Salva e chiudi editor
# Modifica il commit message finale se necessario

# Force push (solo dopo squash)
git push origin feat/openai-compatible-transcription -f
```

---

## 📊 Passo 6: Monitoring e Follow-up

### Dashboard da Monitorare

1. **PR su GitHub**:
   ```
   https://github.com/zeroclaw-labs/zeroclaw/pull/<PR_NUMBER>
   ```

2. **CI Status**:
   - Actions tab → CI Required Gate workflow
   - Deve essere verde prima del merge

3. **Review Status**:
   - Required: 1 maintainer approval (Track B)
   - CODEOWNERS auto-assigned per `src/channels/**` e `src/config/**`

4. **Merge Blockers**:
   - CI non green
   - Review non approved
   - Merge conflict con `dev`
   - Requested changes non risolte

### Timeline Attesa

- **Track B (medium risk)**: 2-5 giorni per review iniziale
- **Follow-up rounds**: 1-2 giorni per re-review
- **Final merge**: Quando tutti checks green + approval

### Se la PR viene Stale

Se dopo 7+ giorni nessuna risposta:

1. **Ping gentile nei commenti**:
   ```markdown
   @maintainers Friendly ping - this PR is ready for review. 
   All CI checks are green and validation evidence is provided.
   Let me know if any additional information is needed.
   ```

2. **Check per duplicates**: Cerca PR simili che potrebbero aver già risolto

3. **Rebasa su dev aggiornato** se è passato molto tempo:
   ```bash
   git fetch upstream
   git rebase upstream/dev
   git push origin feat/openai-compatible-transcription -f
   ```

---

## ✅ Passo 7: Post-Merge Actions

Quando la PR viene mergiata:

1. **Sync il tuo fork**:
   ```bash
   cd C:\Users\marco\Progetti\radosoft\zeroclaw-fork
   
   # Add upstream se non presente
   git remote add upstream https://github.com/zeroclaw-labs/zeroclaw.git
   
   # Fetch latest
   git fetch upstream
   
   # Update local dev
   git checkout dev
   git merge upstream/dev
   git push origin dev
   
   # Delete feature branch (optional)
   git branch -d feat/openai-compatible-transcription
   git push origin --delete feat/openai-compatible-transcription
   ```

2. **Chiudi linked issues** se presenti e non auto-closed

3. **Update working project**:
   ```bash
   cd C:\Users\marco\Progetti\radosoft\zeroclaw
   
   # Update Dockerfile to use upstream main/dev instead of fork
   # Se prima avevi:
   # RUN git clone --branch feat/openai-compatible-transcription https://github.com/drinky78/zeroclaw.git .
   
   # Cambia in:
   # RUN git clone --depth 1 https://github.com/zeroclaw-labs/zeroclaw.git .
   
   # Rebuild
   docker-compose build --no-cache zeroclaw
   docker-compose up -d zeroclaw
   ```

4. **Verifica funziona in produzione** con la versione mergiata

---

## 🎉 Celebrazione e Next Steps

Una volta mergiata la tua prima PR a ZeroClaw:

1. ✅ **Hai contribuito a un progetto open source significativo!**
2. ✅ **Il tuo nome è nel CONTRIBUTORS file**
3. ✅ **Contributor tier badge aggiornato** (auto-managed dopo 5+ PR merge)

**Follow-up contributions**:
- Documentation: Guida setup locale Whisper
- Testing: Integration test per tutti i channel types
- Feature: Streaming transcription support
- Performance: Benchmarking transcription latency

**Rimani coinvolto**:
- Watch il repo per notifiche su issues/PR
- Commenta su issues correlate alla tua expertise
- Aiuta altri contributor con domande su transcription
- Proponi miglioramenti incrementali

---

## 📞 Supporto e Domande

Se hai dubbi durante il processo PR:

1. **Controlla CONTRIBUTING.md** per chiarimenti policy
2. **Cerca PR simili mergiata** per esempi
3. **Chiedi nei commenti della PR** - maintainer sono disponibili
4. **Discord/Slack ZeroClaw** (se disponibile) per discussioni informali

**Non esitare a chiedere aiuto** - è il tuo primo contributo e i maintainer lo capiscono!

---

## 🚨 Cosa NON Fare

- ❌ Aprire PR contro `main` branch (usa `dev`)
- ❌ Pushare senza aver eseguito `fmt`/`clippy`/`test`
- ❌ Includere modifiche non correlate nella stessa PR
- ❌ Committare secrets o API keys reali
- ❌ Force push dopo review è iniziata (a meno che richiesto)
- ❌ Chiudere e riaprire PR per "rifare"
- ❌ Ping maintainer ogni giorno (sii paziente)
- ❌ Modificare file auto-managed (CHANGELOG, size label, etc.)

---

## 📋 Quick Command Reference

```bash
# Verifica stato PR-ready
cd C:\Users\marco\Progetti\radosoft\zeroclaw-fork
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
git status

# Apri PR (web)
# https://github.com/drinky78/zeroclaw → "Compare & pull request"
# Base: zeroclaw-labs/zeroclaw @ dev
# Head: drinky78/zeroclaw @ feat/openai-compatible-transcription

# Apri PR (CLI)
gh pr create --repo zeroclaw-labs/zeroclaw --base dev \
  --title "feat(channels): add OpenAI-compatible transcription provider support" \
  --body-file PR_DESCRIPTION.md

# Fix review feedback
git add .
git commit -m "fix: address review comment - <what>"
git push origin feat/openai-compatible-transcription

# Squash commits
git rebase -i HEAD~2
git push origin feat/openai-compatible-transcription -f

# Post-merge cleanup
git checkout dev
git fetch upstream
git merge upstream/dev
git push origin dev
git branch -d feat/openai-compatible-transcription
git push origin --delete feat/openai-compatible-transcription
```

Buona fortuna con la tua prima Pull Request a ZeroClaw! 🚀
