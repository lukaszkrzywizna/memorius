# Plan 0002 — Agent-declared tickets and parallel splits

Status: accepted by Łukasz 2026-10-15. Revised the same day after Codex's
review (eligibility, privacy precedence, stale splits, declaration order).
Not started.

Spec: `docs/product-specs/memorius/spec.md`. Architecture: `ARCHITECTURE.md`.
Screens: the private design canvas ("Split your time
between tickets (planned)", living prototypes on "Wild looks").

## Why

Inference guesses the ticket from titles, URLs, branches and commits. Real
data shows where it fails: the branch `claude/is61-uuid-keys-pg` has no full
key, a research session has no key at all, and private sessions run in the
work repository's folder, so folder rules cannot separate private from business work.
The agent knows its ticket; it should say so.

## Rules

1. A declaration decides only **which ticket**. It is never evidence of the
   user. _When_ and _how much_ stay as today: presence (app rows, prompt
   minutes, short breaks) and agent time up to 30 minutes after a prompt
   (`TimeSplit`, plan 0005).
2. A declaration covers its own session from `effectiveFrom` until the next
   declaration, `stop`, or the session's last activity, intersected with the
   session's own activity: its runtime spans and its prompt minutes. It
   never fills quiet gaps and never claims another session's minutes or an
   overlapping app interval.
   Example: prompt 10:00, agent works 10:00–10:20, quiet 10:20–11:00, prompt
   11:00. The declaration covers 10:00–10:20 and 11:00 onward, not the quiet
   gap.
3. A session without a declaration is **unknown**, not private and not
   "not business". Unknown time is never proposed automatically. It stays
   visible in the Agents lane, and inference shows its candidate as a hint.
4. Provenance is explicit: `agentDeclared` is different from `inferredHigh`.
5. Explicit `unknown` and `private` declarations are allowed. `private` keeps
   the session out of proposals.
6. Precedence: an explicit privacy decision or correction by the user beats
   any agent declaration. A later agent declaration cannot undo it. (Today
   there is no way to exclude private sessions or apps from worklog minutes;
   see `ARCHITECTURE.md`. This plan adds it.)

## Eligibility, then allocation

Allocation runs on exact ranges in two steps, in one pure allocator used by
Day, `unlogged` and proposals.

1. **Eligible intervals per session**: the session's own prompt minutes, plus
   its runtime up to 30 minutes after them, minus anything the user marked
   private. Unrelated app activity does not certify a session: a private
   prompt 10:00–10:01 next to an unattended session declared A gives A zero.
2. **Allocation**: for every eligible minute, the tickets of the sessions that
   are eligible in that minute share it. `ticket allocations + unallocated =
eligible budget`. Private and unknown shares go to _unallocated_; they are
   never redistributed to the other tickets.
3. App intervals keep their own attribution (rules, inference). They are not
   removed by an overlapping session, and a session does not take them over.
4. Intervals already in approved or logged proposals are reserved. A new
   declaration or split never creates a second proposal for the same interval.

## Identity

- The MCP server stores the declaration itself. It does not parse session logs.
- Claude Code: the server reads `CLAUDE_CODE_SESSION_ID` from its environment.
- Codex: no id in the MCP environment. The agent passes `source` + `sessionId`
  explicitly (the skill tells it to read `CODEX_THREAD_ID` with `printenv`).
- Without a verified or explicit id the call fails visibly. Never guess from
  cwd, the newest file, or time.
- The id is checked against imported session metadata. If the session is not
  imported yet, the declaration is stored as **pending** with no effect on
  proposals, and the response says "pending import", not "saved".
- Subagents and forks have their own identity. A subagent never changes its
  parent's declaration.

## Order and replay

- Every call carries an idempotency key; the same key twice is one record.
- A live declaration takes effect at the server's receive time. Per session,
  declarations are ordered by that time and a sequence number, and storing a
  new one closes the previous one in the same transaction.
- A late or retried call cannot move the session back: a live declaration
  older than the session's current one is rejected with a visible error (use
  a backfill instead).
- A pending declaration keeps its original effective time and stays unknown
  until the session is imported and validated; it does not start at import
  time.

## MCP and CLI

- `memorius_working_on { issue | "unknown" | "private", source?, sessionId? }`
  validates the issue key and returns the stored declaration with the identity
  it used.
- `memorius_stop_working { source?, sessionId? }` ends the current declaration.
- `memorius_unlogged` and `memorius_day` report undeclared agent time per
  session so the reporting agent can ask: "some entries seem missing, please
  review these sessions".
- `memorius_declare_backfill { source, sessionId, from, to, issue }` creates a
  retroactive declaration only after the user confirms it in the app (see
  Backfill).

## Backfill

- Exact source, session and half-open range only.
- Stored with `recordedAt`, `effectiveFrom`, `effectiveTo`, provenance
  `live | backfill | userCorrection` and the confirmation reference.
- Needs the user's confirmation in the app. Text from a model is not approval.
- Never changes approved or logged worklogs, and never changes supervision
  answers.
- Idempotent: the same backfill twice is one record.

## Parallel tickets

- When sessions with different tickets overlap, a minute counts once. One
  global budget is split between the tickets.
- Default split is equal. The user can change the percentages per stretch in
  the app ("Change split").
- A split answer stores the candidate set it was made for (tickets and
  sessions) and a revision of it. If the set changes later (a new session is
  imported, one becomes private), the answer becomes **stale**: the app shows
  it, unapproved proposals made from it are invalidated, and nothing is
  renormalized silently.
- Allocations are stored at millisecond precision. Per ticket and day the
  minutes are rounded once with largest-remainder rounding against the
  whole-minute budget, with a stable tie-break. Rounding never creates
  minutes.
- Unknown and private portions stay out of the split.
- Editing a split keeps the parts of older answers outside its range (same
  rule as `decide_agent_time`). Approved or logged proposals do not change.

## Data

- Migration 8: table `declaration` (id, source, session_id, issue nullable,
  kind `ticket | unknown | private`, effective_from_ms, effective_to_ms
  nullable, recorded_ms, seq, idempotency_key unique, provenance,
  confirmation_ref nullable, status `pending | active`).
- Migration 9: table `split_answer` (from_ms, to_ms, shares JSON of
  issue → basis points, candidates JSON, candidates_revision, decided_ms).
- Private sessions already exist (plan 0005, table `private_session`);
  private apps may add a similar table.
- Generated TS bindings for the new types.

## Phases

1. **Core**: declaration store, resolution (declared range ∩ session activity),
   provenance in `WorkSpan`, unknown/private handling. Inference kept as
   candidate only for undeclared time.
2. **MCP/CLI**: tools above, identity rules, pending state, skill text for
   Claude Code and Codex (`skills/memorius-worklogs/SKILL.md`, `docs/agents.md`).
3. **Split**: overlap detection, equal default, split answers, rounding in
   `unlogged` and proposals.
4. **UI**: declared badge and provenance on agent blocks and work spans,
   undeclared hint, split control on parallel stretches, backfill
   confirmation. Uses the shadcn + glass components.

Each phase ends with green `pnpm agent:preflight` and a commit.

## Acceptance tests

- Two sessions share one repository folder; one declares A, one stays
  undeclared: only A's own evidence goes to A, overlapping human minutes count
  once, the undeclared session's prompts do not certify A, unconfirmed runtime
  stays unconfirmed.
- A/B overlap conserves the union; two simultaneous prompt windows do not
  double-count; three sessions on A + one on B split equally by ticket.
- Zero eligible work with hours of agents gives zero allocation.
- Changing the zoom or fragmenting identical intervals does not change results.
- Stale declaration after a ticket switch, crash without stop, restart, fork,
  subagent, replay of the same call, midnight, and two simultaneous sessions.
- Missing identity fails visibly; a declaration for a not-yet-imported session
  is pending and has no effect.
- Backfill without confirmation has no effect; with confirmation it applies
  only to its range; approved and logged proposals stay unchanged.
- 0/100 splits, invalid totals and stale candidate sets are visible errors;
  rounding is deterministic and never creates whole minutes.
- A private prompt 10:00–10:01 next to an unattended session declared A gives
  A zero; allocations + unallocated equal the eligible budget.
- The user marks a session private, then the agent declares a ticket: the
  session stays private. Overlapping private app + business session.
- A split saved for A/B goes stale when C is imported or B becomes private;
  a logged A share stays reserved when B is found later.
- Declarations A → B → replayed A: the session stays on B, the boundary stays
  at B's time. Concurrent calls, replayed stop, pending → imported keeps the
  original effective time.

## Out of scope

- Reading declarations from session log text.
- Hooks or global agent instruction changes (each needs the user's consent).
- Prompt text display: a separate small task, see below.

## Related decision: prompt text on hover (2026-10-15)

Decided by Łukasz; a separate small task, not part of the phases above.

- The importer stores, per prompt, only its time and the byte offset of its
  line in the session file. No prompt text goes into the database.
- On hover, a Tauri command reads that one line from the local Claude Code or
  Codex session file and returns the text. A missing or moved file shows
  "Text no longer available".
- The app shows prompt text for all sessions, private ones too: the data is
  local and belongs to the user.
- MCP and the CLI never return prompt text, for any session, so an agent can
  never copy a prompt into a worklog.
