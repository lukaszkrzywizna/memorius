# Memorius

## Purpose

Memorius is a local activity tracker for macOS and Windows. It records which
apps you use and what you work on in them, so that coding agents can propose
YouTrack worklogs from real evidence. You approve each proposal; then Memorius
logs it.

It must use little memory while it runs all day.

## Why another time tracker

General time trackers record the app and window title. For agent work this is
not enough: Claude and Codex rows only say `Claude` and `ChatGPT`, and browser
rows often have a title but no URL. An agent cannot find the issue.
Memorius adds work context: the session folder and branch of Claude Code and
Codex, the git branch, the browser URL, and calendar events.

## Users and goals

- The user: sees a day and week timeline, fixes context, approves or rejects
  worklog proposals.
- Agents (Claude Code, Codex): read activity and propose worklogs; Memorius
  logs the approved ones to YouTrack.

## Scope (MVP)

1. **Tracking** on macOS and Windows: foreground app, window title, document
   path when exposed, idle, screen lock and sleep. Recording continues when the
   window is closed.
2. **Work context**: issue keys and repos from
   - Claude Code session files (`~/.claude/projects/**`),
   - Codex session files (`~/.codex/sessions/**`),
   - git branch and commits of the session or editor folder,
   - browser URL for supported browsers,
   - user rules (for example `host → issue`, `calendar title → issue`).
3. **Calendar**: Microsoft 365 calendar through Microsoft Graph (`event`
   objects).
4. **Agent interface**: a `memorius` CLI with JSON output and an MCP server
   (`memorius mcp`) over the same data.
5. **Worklog proposals**: agent proposes → user approves or rejects in the app →
   Memorius logs the approved items to YouTrack. Memorius reads the user's YouTrack work items with a token kept
   in its encrypted vault (key in the system keychain), so Week shows real logged time, and logs approved
   proposals itself with it. Memorius can start the
   agent itself (Claude Code, Codex or a custom command, on the user's own
   subscription): on a button, and every day at a chosen time.
6. **UI**: Day timeline (Apps, Agents and Work context columns, zoom),
   Week timesheet, Log queue, tray/menu bar popover, Settings (permissions,
   private apps, rules, calendar sign-in).

## Out of scope (MVP)

- Screenshots, keystrokes, clicks, document or message contents.
- Cloud sync, team features, accounts.
- Memorius writing to YouTrack directly.
- Auto-approve (designed as a later policy switch, off by default).
- Linux.

## Rules

- All data stays on the machine. The only network call is Microsoft Graph for
  the calendar.
- A proposal can be logged only after the user approves it. Approval is stored
  with time and the approved values; the agent must log exactly those values.
- An app marked private records only "Private" time, no title or URL.
- Agent runtime is kept apart from the user's work. Agent time counts on its
  own only up to 30 minutes after the user's prompt in that session (also
  from a phone); later runtime is shown as not counted. The user never
  answers per minute: a whole session can be marked private, and then none
  of it counts.
- When a permission is missing (macOS Accessibility), recording
  continues with less detail, and the UI says so. Missing data is never shown
  as an empty title.
- Memory budget: < 60 MB private memory (Activity Monitor "Memory", Windows
  private working set) with the window closed after 8 hours of recording; the
  WebView is destroyed when the window closes.

## Proposal states

```text
proposed --approve--> approved --Memorius logs--> logged (youtrackWorkItemId)
proposed --reject--> rejected
approved --logging fails--> approved (lastError set, retried on the next log)
```

Duplicate protection: one proposal per (issue, date, time range); a logged
proposal cannot be logged again.

## Acceptance criteria

1. With Memorius running for a work day on macOS and on Windows, the Day
   timeline shows each app switch with correct start and end within 2 s, and
   idle periods as Away.
2. Codex and Claude Code activity shows the issue key from the branch or thread
   title in the Work context lane.
3. Browser time in Chrome, Edge and Safari shows the URL host and path. Dia and
   Arc show the URL through Accessibility; Memorius never asks for a
   per-browser Automation permission.
4. `memorius unlogged --week 2026-W41 --json` returns unlogged blocks with
   evidence; the MCP tool `propose_worklog` creates a proposal visible in the
   Log queue within 5 s.
5. An approved proposal logged from the Log queue shows as logged in the Day
   and Week views.
6. Calendar events from Microsoft 365 show in the Calendar lane after sign-in.
7. The memory budget above holds in an 8-hour soak test on both platforms.

## References

- Architecture: `ARCHITECTURE.md`
- Plan: `docs/exec-plans/active/0001-foundation-and-tracking.md`
