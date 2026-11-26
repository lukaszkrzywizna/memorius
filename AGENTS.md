# Memorius agent guidance

Memorius is a local activity tracker for macOS and Windows (Tauri 2 + Rust +
React). Read before changing code:

- Product spec: `docs/product-specs/memorius/spec.md`
- Architecture: `ARCHITECTURE.md`
- Active plan: `docs/exec-plans/active/`
- Screen reference: the private design canvas (the maintainer has the link)

## Commands

- Install: `corepack enable && pnpm install` (also installs the Git hooks).
- Run the app: `pnpm dev`. UI only: `pnpm -C apps/desktop dev`.
- Storybook: `pnpm -C apps/desktop storybook`.
- All gates: `pnpm agent:preflight` (pre-push; CI runs the fast lanes on Linux per push and PR, all lanes on macOS + Windows weekly when main changed or by hand). Quick: `pnpm agent:preflight:fast`
  (pre-commit, staged files). Logs: `.preflight/<lane>.log`.
- Format: `pnpm format`.
- Memory soak (release build, hidden): `node tooling/soak/soak.mjs --binary target/release/memorius-desktop --minutes 30`.
- Install on this Mac: `pnpm install:mac` (signed with the local "Memorius Local
  Signing" certificate so macOS keeps permissions; never run `pnpm dev` while
  editing crates: it restarts the app on each change).
- Windows/macOS test binaries: run the `Build app` workflow (manual).

## Rules

- Finish and verify the task; run `pnpm agent:preflight` before handing off,
  or say exactly why it was skipped.
- Recording runs in-process. Do not add sidecar processes, local HTTP servers,
  or polling by spawning processes (`osascript`, PowerShell).
- Memory is a feature: close window = destroy the WebView; bounded in-memory
  state; tail files by offset. Check the soak test for recorder changes.
- Never record keystrokes, clicks, screenshots, or document/message contents.
  Missing permissions degrade to less detail, never to fake empty values.
- Memorius reads YouTrack work items with a token from its vault (encrypted
  in the database, key in the keychain; never in logs or prompts) and logs
  approved worklogs itself; agents only propose.
- Frontend: Feature-Sliced Design layers `routes`/`app` → `pages` → `entities`
  → `shared`; a layer imports only lower ones, a slice never a sibling slice,
  and other layers reach a slice through its `index.ts`
  (`apps/desktop/.dependency-cruiser.cjs` checks it). Kebab-case file names, a
  Storybook story (`Apps/Desktop/...`, `autodocs`, one `visual-smoke`) for
  every UI state. Lint: `.oxlintrc.json` and `eslint.config.js`; fix the code,
  not the rule. A disable comment needs a concrete reason.
- Branch on unions with exhaustive `switch`.
- Rust: `cargo clippy -D warnings` with pedantic lints; no `unwrap`/`expect` on
  runtime data paths; platform code behind `#[cfg(target_os = ...)]` in
  `memorius-sources`.
- UI changes that alter layout or interaction: check every state in
  Storybook at desktop width and at 720 px, with a clean browser console.
- Prefer stdlib and platform APIs over new code and dependencies.
- Keep `CLAUDE.md` symlinked to this file.
- Commits: imperative subject, no ticket ID needed. Do not commit unrelated
  changes; never force-push `main`.
