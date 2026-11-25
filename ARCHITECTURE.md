# Memorius architecture

## Shape

One desktop process per user session. No sidecars, no local HTTP server.

```text
Memorius (Tauri 2, Rust)
|- recorder        foreground app, window, idle, lock, browser URL (in-process)
|- context readers Claude Code + Codex session files, git, calendar
|- store           SQLite (WAL) in the app data folder
|- tray / menu bar always present; recording never depends on the window
`- window          React UI; WebView created on open, destroyed on close

memorius (CLI, same Rust workspace)
|- day / week / unlogged ... --json   read-only queries
`- mcp                                MCP server over stdio for agents
```

Why this shape: the previous attempt ran Tauri + a self-contained .NET agent +
a Swift collector with HTTP and an outbox between them. Three runtimes cost
memory before any data arrives. A native tracker can run in about 30 MB; the
recorder here targets that class.

## Workspace layout

```text
crates/
  memorius-core/     model, issue-key detection, interval and context logic,
                     SQLite schema + queries (pure logic is I/O-free and tested)
  memorius-sources/  OS recorder (macos.rs, windows.rs), session readers,
                     git, browser URL, Microsoft Graph calendar
  memorius-cli/      `memorius` binary: JSON queries + `mcp` subcommand
apps/desktop/
  src-tauri/         Tauri host: tray, window lifecycle, commands, scheduler
  src/               React UI (Feature-Sliced Design)
tooling/             quality harness: lint rules, architecture checks, preflight
docs/                product specs, exec plans, references
```

Start with these four units. Split further only when code proves it needs it.

## Recording

The recorder thread reads the foreground state once per second with
in-process API calls (ActivityWatch does the same). This is simpler than
event hooks, behaves the same on both platforms, and is accurate to about a
second. No process spawning (`osascript` and similar are never used).

| Signal                 | macOS                                                                                     | Windows                                                  |
| ---------------------- | ----------------------------------------------------------------------------------------- | -------------------------------------------------------- |
| Foreground app         | `NSWorkspace.frontmostApplication` (`objc2-app-kit`)                                      | `GetForegroundWindow` + `GetWindowThreadProcessId`       |
| Window title, document | Accessibility `AXFocusedWindow` → `AXTitle`, `AXDocument` (0.25 s cap)                    | `GetWindowTextW`                                         |
| App identity           | bundle id, localized name                                                                 | `QueryFullProcessImageNameW` + version `FileDescription` |
| Idle                   | `CGEventSourceSecondsSinceLastEventType`                                                  | `GetLastInputInfo`                                       |
| Lock                   | `CGSessionCopyCurrentDictionary` → `CGSSessionScreenIsLocked`                             | `OpenInputDesktop` fails while locked                    |
| Sleep, app not running | a gap of more than 15 s between readings ends the open row                                | same                                                     |
| Browser URL            | Accessibility: page web area `AXURL` or address field (Chromium: `AXManualAccessibility`) | UI Automation: address bar of Chrome / Edge              |

Without macOS Accessibility, app, idle and lock still record; title is marked
`unavailable(permissionDenied)`, never an empty string.

Storage follows the ActivityWatch heartbeat model: a new snapshot equal to the
open row extends its `end`; a different snapshot closes it and opens a new row.
Rows are written at most every 5 s, so a crash loses at most 5 s.

## Data (SQLite)

Schema version = number of migrations in `memorius-core/src/store.rs`; never
edit an applied migration, add a new one (SQLite changes a CHECK only by
rebuilding the table, as migration 4 does).

- `activity` — kind (`app` | `away` | `locked`), app id and name, title or the
  reason it is unavailable (`permissionDenied` | `noWindow` | `private`),
  document path, URL (scheme, host and path only), start, end. CHECK
  constraints keep app columns and title states consistent.
- `session_file` — read position and last known state (session id, folder,
  branch, title, helper flag) per Claude Code / Codex session file.
- `context_span` — stretches of one agent session (pause under 15 minutes,
  one branch) with the times the user typed a prompt. Rebuilt from the
  session files; safe to drop (migration 3 did).
- `rule` — private apps and issue rules (app, website host, title text).
- `proposal` — one issue on one local `date` (trackers log per issue and
  day, with no start or end time), minutes, text, work type and state; its
  start and end are the work behind it, as evidence. One waiting proposal
  per issue and day (migration 14 merged older ones).
- `tracker_worklog` — the user's own YouTrack work items of the last 90
  days (issue, day, minutes); a sync replaces its range.
- `private_session`, `agent_run`, `setting` — see Day blocks and Agent
  interface.
- `secret` — vault secrets, encrypted (see Agent interface).
- Later: `calendar_event`.

The desktop app writes `activity` (recorder thread), the session tables
(importer thread, once a minute) and `rule` (Apps & rules screen). The CLI
writes only new proposals (`propose`); Memorius itself marks them logged or
failed when it logs them. WAL + `busy_timeout` make this safe. Read-only
opens refuse a newer or older schema version and say so.

## Work context

Readers in `memorius-sources`, logic in `memorius-core::context`:

- Claude Code (`~/.claude/projects/*/*.jsonl`, and helper agents in
  `*/<session>/subagents/*.jsonl` with the parent's session id): `cwd`,
  `gitBranch`, session title, and user prompts (not tool results, side
  chains or meta lines).
- Codex (`~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl` and
  `~/.codex/archived_sessions/`): files are known by name, because
  archiving moves them; folder from the session header; thread names from
  `session_index.jsonl`; review threads
  (`thread_source: guardian_review`) are skipped; helper agents count for
  their parent thread but never as prompts; replayed history in forked
  threads is ignored.
- Overlapping spans of one session (a helper agent's file next to its
  parent's) join, so their time counts once.
- Git: branch at any time and commit subjects from `.git/logs/HEAD`
  (worktrees included), cached until the file changes.
- Browsers: active tab URL. macOS reads it through Accessibility (the
  page's web area or the address field), so no per-browser Automation
  permission is ever asked; the element is found once per window (at most
  300 elements) and only read afterwards; a new tab title searches again.
  Windows uses UI Automation.

## Day blocks

The day is cut into slots from local midnight: 15 minutes by
default, and 1, 5, 10, 30 or 60 minutes through the Day view's zoom (every
slot is drawn 32 px tall). Each slot takes its dominant window (app + title, by time); offline time (away or
locked) wins only when it is more than three times the top window's time,
because our idle time starts at the last input. Neighbour slots with the
same dominant form one block, which lists every window inside it. Empty
slots split blocks. Day totals come from the raw rows, not from blocks.
A block or work span is drawn on the slot grid, but its minutes are the
active time inside it (app use or agent work, counted once): one minute in
a slot is one minute of work. A proposal covers its minutes, spread over
the active time in its range, so a short worklog in a long range leaves the
rest unlogged.

The Agents column shows when Claude Code and Codex sessions worked: each
session stretch widens to whole slots, stretches of one session (same agent
and title, so resumed conversations join) that touch form one block, and
parallel sessions sit side by side. The column shows all agent runtime.

Agent runtime is not the user's work by itself. All accounting uses exact
time ranges, so the display zoom never changes it
(`memorius-core::timeline::account`):

- Presence: app time on the Mac and each prompt's own minute (prompts from
  a phone are in the same session logs), with gaps of up to 15 minutes
  between them filled (reading, thinking). A longer gap is a break.
- Agent time outside presence counts up to 30 minutes after a prompt of the
  same session (`TimeSplit.agentMs`). Later runtime, and sessions with no
  prompt (heartbeats, schedules), do not count (`uncountedMs`). A prompt
  never certifies another session.
- The user's work (`TimeSplit.workMs`, the ranges worklogs cover) =
  presence ∪ counted agent time, counted once. Nothing asks about minutes.
- The user can mark a whole session private (table `private_session`): its
  prompts and agent time do not count and it names no issue; it stays in
  the Agents lane. App time in the same range still counts.

The windows come from two weeks of history (October 2026): real reading
pauses were 3–16 minutes after a prompt, and the only longer agent-only
runs were a heartbeat session. The older per-minute answers (table
`agent_decision`) are kept but no longer read.

Memorius cannot see meetings or work away from the Mac without a prompt;
that time is not counted (the calendar will add meetings).

Known gap: private apps hide titles but are still counted.

## Issue detection

Pure functions in `memorius-core`, table-tested:

Work is detected for every slot with app time or agent time.

1. Uppercase issue keys (`INT-290`) in window titles, URLs, session titles and
   commit subjects; keys in any case only at the start of a branch path
   segment (`claude/int-290-...`), so `batch-2` is never a key. Standards such
   as `UTF-8` and `SHA-256` are excluded.
2. Each slot with app time scores its candidates: window title, URL and rule matches
   by their time; agent sessions by their overlap, fully when the user typed
   a prompt in the slot, a quarter when the session only ran in the
   background. A session's keys come from its branch, its title and every
   commit made in its folder while it ran that day, so one commit names all
   of the session's time. "Title contains" rules also match session titles
   and branches.
3. Each span reports its evidence `support`: the issue's share of the
   span's detection weight (named or not; typed sessions count fully,
   background ones a quarter). It is not a share of minutes; minutes come
   from the time split. Other issues are listed with their support.
   Confidence is `high` only when a title, URL, rule or typed session
   names the issue, the issue holds at least half of the slot's weighted
   time (named or not), and no other issue has half of its weight. A short
   URL among long unnamed work, or a close second issue, gives `medium`: a
   suggestion. A span is `high` when high slots hold at least half of its
   active time. Neighbour slots
   about the same issue form one work span; spans do not follow the app
   blocks and carry the evidence (the Work context lane and its card).
   Nothing is guessed from free text.

## Agent interface

`memorius` (crate `memorius-cli`) prints JSON with local ISO times; `memorius
mcp` serves the same operations as MCP tools over stdio (a small hand-written
JSON-RPC loop: `initialize`, `ping`, `tools/list`, `tools/call`). Both share
the read models in `memorius-sources::views` with the desktop app. Guide and
skill: `docs/agents.md`, `skills/memorius-worklogs`.

Proposal states (`memorius-core::proposal`):

```text
proposed --approve (with edits)--> approved --Memorius logs--> logged
proposed --reject--> rejected
approved / rejected --undo--> proposed     (until logged)
approved --agent fails--> approved + lastError
```

One proposal per issue and time range (`UNIQUE`); proposing again returns the
existing one. Agents log to YouTrack with their own access.

Memorius reads the user's YouTrack work items itself (plan 0007): the
address in `setting`, a permanent token in the vault.

The vault (`src-tauri/src/vault.rs`) keeps secrets in the
`secret` table encrypted with ChaCha20-Poly1305 (random nonce, the name as
associated data, so a ciphertext cannot move to another name). Its one key
lives in the system keychain (`keyring`, item "vault-key"), made on first
use and read once per run. Secrets never reach logs, prompts or the CLI. `memorius-sources::youtrack` reads
`/api/workItems?author=me` for the last 90 days on start, every 15 minutes
and on request (`src-tauri/src/tracker.rs`); a failed sync keeps the last
good data. `week::DayLedger` is the one per-day account: logged (YouTrack,
plus agent-logged proposals not synced yet), proposed and detected minutes
per issue, and not logged = the user's work − logged − proposed. Week, the
CLI and MCP (output version 9, per day) use it.

With a token, "Log to YouTrack" in the Log queue header (apart from the
agent bar) makes Memorius log the approved proposals itself (`log_approved`): one `POST /api/issues/{id}/timeTracking/workItems`
per proposal with its day (midnight UTC), minutes, text and work type (by
name from `admin/timeTrackingSettings/workItemTypes`). A worklog YouTrack
already holds with the same issue, day, minutes and text is only marked
logged, so a retry or an agent's earlier run never logs twice; a failure
stays approved with its error. It refuses while an agent run is going.
Each sync also stores the instance's work type names (`trackerWorkTypes`):
the agent's prompt names them, `propose` refuses any other (and takes the
tracker's spelling), and the edit form offers only them.
Without a token the button opens Settings to connect YouTrack.

The Commits lane shows the user's own commits on this Mac
(`memorius-sources::commits`, `src-tauri/src/commits.rs`, every 5 minutes).
Repositories come from a root folder in Settings (default
`~/Developer/repos`, searched three levels deep) and from the folders seen
in the last 90 days (agent session folders, documents open in windows);
a repository and its worktrees count once (by the shared git directory).
Commits come from the reflogs of the main checkout and every worktree
(hash, author e-mail, time, subject; never diffs), kept only for the
e-mails from `git config` (global and per repository), with the branch
checked out then; an amended commit counts once. A scan replaces the
`git_commit` table (migration 16), and the day view and `memorius day` read
it. Commits made on another machine or on GitHub do not show.

A daily reminder (`src-tauri/src/reminder.rs`, Settings → Reminder, off by
default) checks every minute and, once a day after the chosen time, shows a
system notification (`tauri-plugin-notification`) with this week's and
today's not logged time from the same ledger; it stays quiet under 15
minutes. "Send a test" shows it at once.

Memorius also starts the agent itself (plan 0006): the Log queue's "Prepare
proposals" button (and "Log to YouTrack" without a token), and an optional daily proposals run at
a chosen time, up to that moment. `memorius-sources::agent` builds the command for
Claude Code (`claude -p … --no-session-persistence`), Codex (`codex exec
--ephemeral --sandbox read-only`) or a custom command line (run with no shell;
`{prompt}` becomes one argument and `MEMORIUS_PROMPT` holds it too). The agent CLI uses
its own login, so the user's subscription pays; `ANTHROPIC_API_KEY` and
`OPENAI_API_KEY` are removed from its environment. A run sees only the
Memorius proposal tools plus the read-only tracker lookups named in settings
(none by default), whose credentials stay in the agent's config.
No session is saved, so a run never counts as the user's agent time.
`src-tauri/src/agent.rs` runs one run at a time in a background thread with
a 15-minute limit and Stop, records it in `agent_run`, and checks the
daily run every minute (a run missed while the Mac slept starts after it
wakes). A Dock-started app has a short `PATH`, so the agent runs with the
login shell's `PATH`. Settings live in the `setting` table, since the tray
process reads them too.

## Calendar

Microsoft Graph `GET /me/calendarView`, delegated `Calendars.Read`, OAuth 2.0
authorization code + PKCE with a loopback redirect, public client (no secret).
The refresh token is stored in the OS credential store (`keyring`: Keychain /
Windows Credential Manager). Sync every 15 minutes and on window open.

## UI

React + TypeScript + Vite, Tailwind, shadcn/ui (Base UI variant), TanStack
Query over typed Tauri commands (`tauri-specta` generates the TS bindings).
FSD layers: `app`, `pages`, `widgets`, `features`, `entities`, `shared`.
Storybook stories for every screen state from the design canvas.

shadcn/ui (Base UI, `base-rhea` style) components live in `src/shared/ui`
and stay as generated, apart from cva variants moved to `*-variants.ts`
(one component per module) and their graphite restyle (pill buttons, normal
weight). Add a component with `pnpm exec shadcn add` when a screen uses it.
The graphite theme (plan 0004) is in `styles.css`: light tokens on `:root`,
dark tokens on `.dark`, mapped onto the shadcn variables plus text steps
`t1`–`t5` and the attention, operational and critical accents; utilities
`glass`, `frost`, `warm-panel` and `ticks*` on a solid backdrop, with solid
surfaces for `prefers-reduced-transparency`. `shared/lib/theme` puts `.dark`
on `<html>` from the System / Light / Dark preference and follows the system
setting and the other window.

The main window shell (`app/shell`) is a top bar: Tick dial logo, page tabs
(Log queue with the waiting count), recorder pill, Needs you button, theme
switch and Settings. `ShellLayout` is the presentational seam with stories;
`AppShell` adds router links and queries. Below 720 px the tabs move to a
bottom bar. Width tiers use viewport media queries (`max-[720px]`,
`max-[960px]`), because the shell always fills the window and popovers are
portaled. Needs you is a popover on every page: waiting proposals and
today's detected issues without a proposal.

The timeline is plain DOM with absolute positioning (no scheduler library).
It runs horizontally (lanes top to bottom, scroll sideways) or vertically
(lanes side by side, scroll down); the choice and the zoom are stored per
device. Lanes: Apps, Agents (parallel sessions share the lane), Work and
Prompts (times only, from `AgentBlock.promptTimesMs`). One details card sits
next to the selection, a bottom sheet below 720 px. `?issue=` dims all but
one issue (used by search). App shortcuts use `@tanstack/react-hotkeys`.

Search (⌘K / Ctrl+K) is a palette in the shell over one read-only store
query (`Store::search`): window titles and addresses, agent session titles
and branches, and proposals, over the last 90 days, with the query
escaped for `LIKE`, row and hit limits, and titles of apps that are private
now left out. Logged and proposed minutes stay apart. Dates are parsed in
the UI. It never searches prompt text, and MCP does not offer it.

Loading states are skeletons in the shape of the content (`Skeleton`,
`LoadingRegion` in `shared/ui`), never a line of text. The app and tray
icons come from `src-tauri/icons/app-icon.svg` and `tray-icon.svg` (the
Tick dial), rendered to PNG with `tooling/render-svg.swift` (transparent
background; Quick Look adds a white one, which turns the template tray icon
into a white square) and passed to `pnpm tauri icon`. macOS 26 and
later put such an icon on a light tile, so the bundle also carries the Icon
Composer icon `icons/AppIcon.icon` compiled into `Assets.car`
(`tooling/build-mac-icon.sh`, needs Xcode) and named by `CFBundleIconName`
in `src-tauri/Info.plist`; keep its dial layer in step with `app-icon.svg`.

## Memory rules

- Close window → destroy the WebView. Tray/menu bar popover is a small window
  created on demand.
- No UI polling while hidden; push changes with Tauri events while open.
- Bounded in-memory state: only the open activity row and the last snapshot.
- Context readers tail files by offset; they never load whole session logs.
- Measure private memory: macOS `phys_footprint` (Activity Monitor "Memory"),
  Windows private working set. RSS counts shared system frameworks and is not
  a useful budget. Measured 2026-10-11 (release, macOS, no window): 17 MB.
- `tooling/soak/soak.mjs` starts the release build hidden and samples private
  memory every minute; it fails over 60 MB or on more than 8 MB growth. An
  8-hour run on each platform is a release check.

## Releases and updates

A `v*` tag runs `.github/workflows/release.yml`: a universal macOS build and a
Windows build into a draft GitHub release. The `memorius` CLI ships inside
(`tooling/build-cli-sidecar.mjs` + `tauri.bundle.conf.json`, Tauri
`externalBin`, next to the app executable); agent runs start it by path, and
Setup → Command line links it as a command and registers it with Claude Code
or Codex.
macOS builds are signed with one self-signed "Memorius Release" certificate
(its key is a GitHub secret; a copy lives in the maintainer's
`~/.memorius-release`). The app's designated requirement pins that
certificate's hash, so Accessibility and other grants survive every update
without an Apple Developer ID; Gatekeeper asks once, at the first launch.
`pnpm install:mac` signs local builds with the same certificate when it is
present. The Tauri updater (`updates.rs`) reads `latest.json` from the latest
release, checks once a day and notifies once per version; Settings → Updates
installs and restarts. Updates are verified with the updater key in
`tauri.conf.json`.

## Setup and lost permissions

Until the user finishes it once (`setupDone` setting), the start route opens
`/setup`: one row per permission or connection (notifications, title access,
start at login, agent, YouTrack), checked every second while open so a row
turns green as soon as the switch flips; only title access is required.
Notifications have no permission API on macOS, so the user confirms the test
notification. Agent sign-in opens a terminal with the CLI's own login
(`claude auth login`, `codex login`); `agent_auth` reads its status. After
setup, the recorder notices title access taken away while it runs: one
notification, a "!" next to the menu bar icon, and a banner on every page
that leads back to setup; a signed-out agent gets its own banner.

## User settings instead of assumptions

- Repository folders (`git.roots`, Settings → Git) are a list, empty by
  default; repositories in folders the user or an agent worked in always
  count, and each scan suggests folders holding several of them.
- Agent session folders (`agent.claudeHome`, `agent.codexHome`) override
  `CLAUDE_CONFIG_DIR` / `CODEX_HOME`, which a Dock-started app cannot see.
- Time rules (`timeRules`, Settings → Advanced): away after no input, breaks
  counted as work, agent time after a prompt, and the first day of the week.
  `views::day_timeline` and `views::week_summary` read them, so every view,
  the CLI and the reminder count the same way.
- Issue keys count only for projects the tracker knows (`trackerProjects`,
  stored at each sync); without a tracker every `ABC-123` counts.
- Clocks follow the system's 12- or 24-hour setting; day and month names
  stay English.
