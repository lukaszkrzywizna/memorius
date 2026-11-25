---
name: memorius-worklogs
description: Use when the user asks to prepare, propose, check or log YouTrack worklogs (time tracking, "log my time", "what did I do this week", unlogged time) from their Memorius activity, or to log worklogs they approved in Memorius.
---

# Memorius worklogs

Memorius records which apps and windows the user worked in, and which
YouTrack issue each block of time was most likely about (from branches, agent
sessions, window titles, URLs and the user's rules). You propose worklogs;
the user approves them in the Memorius app; then you log exactly the approved
values in YouTrack.

Use the Memorius MCP tools (`memorius_*`). Without MCP, run the `memorius`
command line; it prints the same JSON (`memorius --help`).

## Propose

1. `memorius_unlogged` (from/to dates, default this week; `"version": 9`)
   returns the days with at least 5 minutes of the user's work that no
   YouTrack work item or proposal covers. YouTrack logs per issue and day,
   with no start or end time. All minutes are the user's work: app time on
   the Mac, each prompt's minute (also from a phone), breaks of up to 15
   minutes between them, and agent time up to 30 minutes after the user's
   prompt in that session (`agentMinutes`), counted once. Per day:
   - `workMinutes`, `unloggedMinutes`; `logged` (from YouTrack, also time
     logged by hand) and `proposed`, each a list of `issue` and `minutes`;
   - `work`: spans (`start`, `end`, `minutes`, `agentMinutes`, `issue`,
     evidence `support` in % (not a share of minutes), `otherIssues`,
     `evidence`). `issue` is only Memorius's suggestion from the evidence;
   - `blocks`: app blocks (`start`, `end`, `minutes`, `app`, `title`,
     `windows`).
2. Judge the evidence. `memorius_day` shows a day's windows, addresses,
   agent sessions and commits; read the issue in the tracker when you can.
   Decide which issues each day's work belongs to and how many minutes
   each; leave out what was not work on an issue and what is logged.
3. Propose one worklog per issue and day with `memorius_propose_worklog`:
   - `issue`, `date`; `start`/`end`: the first and last moment of that
     work (local times as given); `minutes`: at most the day's unlogged
     minutes in total.
   - `text`: what was done that day, in one or two sentences, from the
     evidence. Do not invent work.
   - `workType`: one of the user's YouTrack work types, or none when none
     fits (Memorius refuses others once it knows them); `agent`: your name.
     Every proposal waits for the user's approval in Memorius.
4. Stop and tell the user how many proposals wait in the Memorius Log queue.
   Never log in YouTrack at this step.

Proposing the same issue and day again while it waits replaces its minutes
and text, so retrying is safe. A rejected proposal stays rejected; do not
work around it.

## Rules

- Never log time yourself: Memorius logs the approved proposals.
- Times are local ISO 8601 (`2026-10-09T13:31:00+02:00`); pass them back as
  they are.
