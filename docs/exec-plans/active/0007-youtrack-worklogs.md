# Plan 0007 — Real YouTrack worklogs, logged per issue and day

Status: accepted by Łukasz 2026-10-16; phases 1–3 built the same day.

## Why

- Week shows as logged only what an agent logged through Memorius. Time
  logged by hand, by another tool or before Memorius is missing, and
  Memorius treats it as unlogged, so an agent can propose it a second time.
- A YouTrack work item has an issue, a day, minutes, text and a work type;
  it has no start or end time. Memorius proposes per time span, so one
  issue gets several small proposals a day (17 proposals for 15–16 Oct,
  five of them SEL-78).

## Decisions (Łukasz, 2026-10-16)

1. Memorius may hold a YouTrack token. It lives in the macOS Keychain
   (Windows Credential Manager later), never in the database, logs or the
   agent's prompt. This replaces "Memorius never holds a YouTrack
   credential" for reading; logging stays with the agent for now.
2. Memorius reads the user's own work items for the last 90 days.
3. Not logged time and the agent's data subtract time already logged in
   YouTrack.
4. The unit of logging is issue × day, as in YouTrack.

## Connector (read only)

- Settings → YouTrack: instance URL and permanent token; a Test button
  shows the user's name. The token goes to the Keychain (`keyring` crate).
- `GET /api/workItems?author=me&startDate=…&endDate=…&fields=id,date,
duration(minutes),text,type(name),issue(idReadable,summary)`; paged.
- Sync on start, when Week opens, and every 15 minutes; it replaces the
  stored items of the synced range. Failures show in Settings and keep the
  last good data.
- Migration 13: table `tracker_worklog` (id, issue, summary, date,
  minutes, text, work_type, synced_ms).

## Issue × day model

- A proposal is one issue on one local day: `date`, `minutes`, `text`,
  `work_type`, plus the spans it came from as evidence. Proposing the same
  issue and day again while it waits updates it (minutes, text); approved
  and logged proposals stay as they are.
- Migration 13 also adds `date` to `proposal` (from `start_ms`) and keeps
  start/end as evidence. Waiting proposals of one issue and day are merged
  (minutes summed, texts joined); approved, logged and rejected ones are
  not touched. The database is backed up first.
- Coverage per day: Not logged = active time − (YouTrack worklogs + waiting
  and approved proposals) of that day, at most the active time.
- Week: a cell is logged (YouTrack, plus Memorius-logged items not yet
  synced), proposed, or detected, per issue and day.
- Agent output version 8: `memorius_unlogged` gives per day the active,
  logged (by issue, from YouTrack) and proposed minutes, the unlogged
  minutes, and the spans with suggested issues as evidence;
  `memorius_propose_worklog` takes `issue`, `date`, `minutes`, `text`,
  `workType`, `agent`. The prompt asks for one worklog per issue and day.
- Log queue rows show the day instead of a time range; the details pane
  lists the spans.

## Phases

1. Keychain token, YouTrack client, sync, `tracker_worklog`; Settings
   section; tests with a fake server.
2. Issue × day proposals: migration with merge, store, coverage, Week,
   CLI/MCP v8, prompt and skill.
3. Log queue and Week UI; docs (`ARCHITECTURE.md`, spec, README).

## Later (done 2026-10-19)

- Memorius logs approved proposals by itself through the same connector
  ("Log approved" with a token); the agent log run stays for users
  without one.

## Notes from the build

- Week opens with the stored data; "Sync now" in Settings refreshes on
  request (no sync on opening Week).
- Week's "active" is now the user's work as on the Day view, and every
  span's suggested issue counts as detected.
- Łukasz chose a private vault for this and more secrets later: one key in
  the keychain, secrets encrypted in the `secret` table (migration 15).
