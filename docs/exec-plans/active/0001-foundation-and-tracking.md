# Plan 0001 — Foundation and tracking

Status: accepted 2026-10-11. Phase 0 done (dbb5da4, CI green on macOS and Windows). Phase 1 in progress.

Spec: `docs/product-specs/memorius/spec.md`. Architecture: `ARCHITECTURE.md`.
Screens: the private design canvas

Each phase ends with a working app, green preflight, and a commit. Phases 1–5
map to the design artboards as noted.

## Phase 0 — Repository and harness

Goal: an empty Tauri app with strict quality gates from day one.

1. Private GitHub repo `lukaszkrzywizna/memorius`, `main` protected by CI.
2. Tauri 2 scaffold: Rust workspace (`crates/*`, `apps/desktop/src-tauri`),
   React + Vite + TS in `apps/desktop`, pnpm workspace (corepack, pnpm 11).
3. Quality harness for one frontend:
   - `.editorconfig`, `.npmrc`, `.oxfmtrc.jsonc`, `.oxlintrc.json`,
     `tsconfig.base.json`, root `eslint.config.js` (FSD boundaries, a11y,
     react-hooks, you-might-not-need-an-effect, testing-library, storybook,
     vitest, TanStack Query plugins);
   - `tooling/frontend-harness/` (eslint FSD rules, semgrep rules,
     dependency-cruiser base, architecture checks: FSD slice usage, flat UI
     segments, component composition, state architecture, effect and
     serialization, Storybook stories) with its own tests;
   - `knip.json`, Vitest + coverage, Storybook;
   - `lefthook.yml` and a small `tooling/harness/preflight.mjs` with
     `--fast` (staged scope) and full modes.
4. Rust gates: `rustfmt` (`--check`), `clippy --all-targets -D warnings`,
   `cargo test`, `cargo deny` (licenses, advisories).
5. GitHub Actions: preflight on `macos-latest` and `windows-latest`.
6. Agent docs: `AGENTS.md` (+ `CLAUDE.md` symlink), `.agents/skills` symlinked
   to `.claude/skills` with the generic skills: feature-sliced-design,
   typescript-variant-switch, storybook-component-driven,
   devtools-validation-loop, ponytail-review, improve-codebase-architecture.

Done when: `pnpm agent:preflight` passes locally and in CI on both OSes.

## Phase 1 — Recording (Day timeline: Apps lane; Menu bar popover)

1. `memorius-core`: `activity` schema + migrations (`rusqlite`, bundled
   SQLite), heartbeat merge, day query. Unit tests with table cases: merge,
   split, idle, lock, clock change, crash recovery (open row closed at last
   write).
2. `memorius-sources` macOS recorder: NSWorkspace activation, AX title and
   document with AXObserver, idle, lock, sleep; permission state.
3. `memorius-sources` Windows recorder: WinEvent hooks, title, process path,
   idle, lock, power.
4. Tauri host: tray / menu bar, autostart at login, close = destroy WebView,
   quit = flush and stop.
5. UI: Day timeline Apps lane, inspector (windows and pages list), totals,
   menu bar popover, Settings → Permissions.
6. Soak harness: synthetic switches, RSS log, CI short run.

Done when: one real work day recorded on macOS; 2-hour recording on Windows;
memory budget holds; Storybook has every Day/popover state from the canvas.

Progress 2026-10-11: items 1–6 built (commits 1788fde, 087a4ff, 4e86af4).
macOS release build, no window: 17–18 MB private memory, flat in a 30-minute
soak; lock and idle verified on a real locked Mac. Windows release build
records correctly in a signed-in desktop (Windows recording check workflow,
run 38160922483: Notepad and File Explorer switches with titles, initial
away time, 2.4 MB private memory). Real macOS use with Accessibility started
the same day (titles from Teams, Slack, Dia, Outlook). Open: a full work day
on macOS, and longer real use on a Windows machine.

## Phase 2 — Work context (Work context lane, inspector "Detected work")

1. Issue detection functions + rules in `memorius-core`.
2. Readers: Claude Code sessions, Codex sessions (tail by offset), git branch
   and commits of the session folder.
3. Browser URL: macOS through Accessibility (changed 2026-10-12 from
   AppleScript, which needed an Automation permission per browser).
   Windows UI Automation for Chrome and Edge.
4. Apps & rules screen: private apps, host/app/calendar → issue rules.

Done when: acceptance criteria 2 and 3 of the spec pass on a real day.

Progress 2026-10-11: items 1–4 built (aa690dd, 86a5dc1, cb790b2, 55739b3).
First import of 90 days of sessions: 919 files in 5 s, then 0 s per minute.
Windows URL capture is checked by the Windows recording check workflow.
Open: a real work day to confirm criteria 2 and 3.

## Phase 3 — Agent interface (Log queue, Week)

1. `memorius` CLI: `day`, `week`, `unlogged`, JSON output with schema version.
2. `memorius mcp`: the same reads + `propose_worklog`, `list_approved`,
   `mark_logged`, `mark_failed`.
3. Log queue screen: approve (with edit), reject, undo; Week timesheet.
4. Agent guide in `docs/agents.md` + a skill for Claude Code / Codex: "propose
   this week's worklogs", "log approved worklogs".

Done when: an agent proposes from a real week, the user approves in the app,
the agent logs to YouTrack and calls `mark_logged`; the Week view matches.

Progress 2026-10-11: items 1–4 built (d53a107 and the guide/skill commit).
Change 2026-10-12 (user): block details open as a floating card next to the
clicked block, not a side panel; app icons come from the OS
through the `appicon://` scheme of the desktop host.

Open: register the MCP server and skill on the user's machine (needs the
user's consent: it changes their agent configuration), then one real
propose → approve → log round with YouTrack.

## Phase 4 — Calendar (Calendar lane)

1. Entra app registration (public client, `Calendars.Read`) — see open question 1.
2. Sign-in (PKCE, loopback), token in OS credential store, calendarView sync.
3. Calendar lane + calendar-title rules.

## Later (not in this plan)

Auto-approve policy, importing history from another time tracker,
Linux, signed installers and auto-update.

## Open questions

1. Resolved 2026-10-11: the calendar connector uses a Microsoft Entra public
   client; the user's tenant must allow user consent (Phase 4).
2. Resolved 2026-10-11: real Windows recording is validated on a Windows VM
   through an interactive RDP session.
3. Code signing: unsigned local builds are enough for personal use; macOS
   permissions stick to a signed app (done 2026-10-12: local certificate).
