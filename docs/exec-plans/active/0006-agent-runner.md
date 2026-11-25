# Plan 0006 — Memorius runs the agent

Status: accepted by Łukasz 2026-10-16. Phases 1–4 built the same day.

## Why

Today Łukasz copies a prompt into Claude Code or Codex to prepare worklog
proposals and later to log the approved ones. Memorius should start the agent
itself, on the agent the user already pays for (Claude or ChatGPT
subscription). Memorius will be open source one day, so the agent is a
setting, not a hard-coded choice.

## Decisions (Łukasz, 2026-10-16)

1. The agent is chosen in Settings: Claude Code, Codex, or a custom command.
2. Proposals: a "Prepare proposals" button and an automatic run once a day,
   the next morning (time in Settings; later "every day at", since 17:30 is
   as valid, and a run goes up to its start). A run covers the time since the last
   run by default; the button also offers 24 h, 48 h, a week and a month.
3. Logging: a "Log approved" button. The agent logs exactly the approved
   values and marks them logged. Nothing logs on approval by itself.

## How a run works

- Memorius starts the agent's own CLI as a child process; the CLI uses its
  own login, so the subscription pays. Memorius never holds a model key or a
  YouTrack credential (YouTrack stays in the agent's MCP config).
- Claude Code: `claude -p <prompt> --output-format stream-json
--no-session-persistence --allowedTools <list>`; proposals also use
  `--strict-mcp-config --mcp-config <memorius only> --tools ""`.
- Codex: `codex exec --ephemeral --json --skip-git-repo-check --sandbox
read-only` with the Memorius MCP server passed by `-c`, and only the
  needed tools enabled.
- Custom: a command line with `{prompt}`; the user owns its tool limits.
- Tools per run kind:
  - proposals: `memorius_unlogged`, `memorius_day`, `memorius_proposals`,
    `memorius_propose_worklog`;
  - log: `memorius_list_approved`, `memorius_mark_logged`,
    `memorius_mark_failed` and the tracker's log-work tool.
- No session is saved (`--no-session-persistence`, `--ephemeral`), so a run
  never shows up as the user's agent time.
- The prompt is fixed by Memorius and names the range; the MCP tool
  descriptions already hold the rules (clear issue only, stop after
  proposing, log only approved values).
- One run at a time. A GUI app has no login-shell PATH, so the agent path
  is resolved once through the login shell and shown in Settings.

## Data

- Migration 12: table `agent_run` (id, kind `proposals | log`, agent,
  from_ms, to_ms, started_ms, finished_ms, status `running | done | failed`,
  summary, error).
- Migration 12 also adds a key-value `setting` table (there is no settings
  store yet): agent, custom command, daily run on or off, run time (default
  08:00), default range. The tray process reads it for the schedule, so it
  cannot live in the WebView's storage.
- "Since the last run" = `to_ms` of the last finished proposals run, at most
  a month back; the first run uses 48 h.

## Schedule

- The tray process checks every minute; at the run time it starts a
  proposals run for the range. A run missed while the Mac slept or the app
  was closed starts at the next start or wake, once.

## UI

- Log queue header: "Prepare proposals" (split button with the ranges) and
  "Log approved (n)"; a run row shows running, done (proposals made, logged)
  or failed with the agent's last message.
- Settings: an Agent section (agent, path check, custom command, daily
  run time, default range).
- The popover shows a running run.

## Phases

1. Runner in Rust (`memorius-sources`): command building per agent, stream
   parsing to a summary, `agent_run` store, tests with a fake agent script.
2. Tauri commands and the Log queue and Settings UI, stories, tests.
3. Daily schedule with catch-up.
4. Docs: `ARCHITECTURE.md`, spec, README for open-source users.

## Risks to check in phase 1

- Codex: per-server tool limits in `-c` and MCP approval in `exec` mode.
- Claude Code: the allowed MCP tool names and that `-p` uses the
  subscription login (no `ANTHROPIC_API_KEY` in the app's environment).
- Long runs: a timeout (default 15 min) and a Stop button.

## Findings (phase 1)

- Claude Code: `-p` with `--mcp-config` (Memorius) next to the user's own
  config works; `--strict-mcp-config` and `--tools ""` leave only the
  Memorius tools. Tested with `memorius_status`.
- Codex 0.162: `-c mcp_servers.<name>.default_tools_approval_mode="approve"`
  lets `exec` call the tool without a prompt; `enabled_tools` limits the
  server; `--ignore-user-config` keeps the login.
- Not done: Windows (`claude.cmd` needs `cmd /C`), a running run in the
  popover.

## Change after the first run (Łukasz, 2026-10-16)

The first run made 0 proposals: the prompt said "propose only spans with
proposable true", and no span reached high confidence. The agent now judges
the evidence: it reads `memorius_day` and, with the read-only tracker tools
in "Issue lookup" (default `youtrack/get_issue,youtrack/search_issues`),
the issues themselves; joins spans of the same work; and proposes
candidates the evidence supports. `proposable` is a hint. The user's
"Instructions" setting is added to the proposals prompt, and Settings shows
the whole prompt. Open: the confidence rule counts windows with no issue
key against every issue, so almost nothing is high.

Then (same day): span issues are only suggestions. Output version 7 drops
`proposable`, `confidence` and `candidate`; a span has `issue` (the
suggestion), `support` and `otherIssues`. The confidence rule is not tuned
further: the agent judges. Later idea, not planned: a YouTrack connector in
Memorius itself, so it can log without an agent.
