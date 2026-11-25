# Memorius

Local activity tracker for macOS and Windows. It records which apps, windows
and coding-agent sessions you work in, guesses the issue behind each block of
time, lets your agent (Claude Code, Codex or your own command) propose
worklogs, and logs the ones you approve to YouTrack.

Everything stays on your computer: one SQLite database, no account, no
server. The YouTrack token is kept encrypted with a key in the system
keychain. Prompt text is read on demand from your agents' session files and
never stored.

## Install

Download the latest release from
[Releases](https://github.com/lukaszkrzywizna/memorius/releases).

### macOS (Apple silicon and Intel)

1. Open `Memorius_<version>_universal.dmg` and drag Memorius to Applications.
2. Open Memorius. macOS blocks it the first time because the app is signed
   with the project's own certificate, not an Apple Developer ID. Open
   **System Settings → Privacy & Security** and click **Open Anyway**.
3. Allow **Accessibility** when Memorius asks: it reads window titles from it.

Memorius checks for new releases once a day; install them from **Settings →
Updates**. Every release is signed with the same certificate, so macOS keeps
the permissions you gave. If you connect YouTrack, the keychain asks for your
password once after each update; choose **Always Allow**.

### Windows

Run `Memorius_<version>_x64-setup.exe`. Windows SmartScreen warns about an
unknown publisher; choose **More info → Run anyway**.

### Agents

Memorius runs your agent (Claude Code, Codex or your own command) to propose
worklogs; pick it in **Settings → Agent**. The `memorius` CLI that agents use
ships inside the app. To use Memorius in your own agent sessions as well,
open **Settings → Setup → Command line**: it installs the `memorius` command
and adds Memorius's MCP server and worklogs skill to Claude Code or Codex. See
[docs/agents.md](docs/agents.md) for what agents can do.

## Build from source

Needs Rust (see `rust-toolchain.toml`), Node.js 24 and pnpm (`corepack
enable`).

```bash
pnpm install
pnpm install:mac
```

`pnpm install:mac` builds Memorius with the CLI inside, signs it with a stable
local certificate and installs it to `~/Applications`.

## Develop

- Product spec: [docs/product-specs/memorius/spec.md](docs/product-specs/memorius/spec.md)
- Architecture: [ARCHITECTURE.md](ARCHITECTURE.md)
- Agent guide and commands: [AGENTS.md](AGENTS.md)
- Quality gate before every commit: `pnpm agent:preflight`
- Releases: bump the version in `tauri.conf.json` and the crates, push, then
  `pnpm release:mac` builds the signed macOS installer (CLI inside) on this Mac
  into a draft release; publish the draft. The Release workflow (by hand)
  also builds Windows.

## License

[MIT](LICENSE)
