# Plan 0005 — Agent time counts on its own

Status: accepted by Łukasz 2026-10-16 and built the same day.

## Why

Łukasz does not review minutes. What matters is the blocks of work to log
and how sure each one is. Answering "count it / not mine" per stretch of
agent time was pointless work; the only answer he needs is "this whole
session is private".

## Evidence (2026-10-12 … 2026-10-16, app data starts 2026-10-11)

- At the Mac (apps + prompt minutes): 2 h 51 min, 4 h 54 min, 5 h 34 min,
  7 h 45 min a day; starts about 08:30–09:15, a 1–2.5 h morning break
  (errands), main block to about 17:00, short evening returns.
- Agent-only time after a real prompt: 3–16 min stretches, starting 1–5 min
  after the prompt; the latest started about 30 min after it.
- The only longer agent-only runs were the scheduled Codex session "Sprawdź
  brak czasu YouTrack" (heartbeats): 35 min to 21 h after its last prompt.
- Pauses of 5–15 min between moments at the Mac (reading, thinking): 4–17
  a day, 43–155 min.

## Rule

- Presence = app time + prompt minutes, with gaps of up to 15 min filled.
- Agent time outside presence counts up to 30 min after a prompt of the
  same session; a session with no prompt never counts.
- Work = presence ∪ counted agent time. `TimeSplit` is `workMs`, `agentMs`
  (counted agent time) and `uncountedMs`.
- A session can be marked private (migration 11, `private_session`): its
  prompts and agent time do not count and it names no issue. Undo counts it
  again.
- The per-minute answers (`agent_decision`) stay in the database unread.

## UI

- Day totals: Your work, Not logged, Agents counted, Not counted.
- Agent and work cards: counted and not-counted agent time per session and
  one "Mark private" / "Count again" button per session.
- Needs you: waiting proposals and unlogged issues only.
- CLI/MCP output version 6: `toConfirm*` and `notWorkMinutes` are gone;
  spans carry `agentMinutes`, the day carries `uncountedAgentMinutes`, agents
  carry `private`.
