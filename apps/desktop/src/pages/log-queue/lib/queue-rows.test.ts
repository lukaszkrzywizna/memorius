import { describe, expect, it } from "vitest"
import type { ProposalState } from "@/shared/api/generated/ProposalState"
import type { QueueItem } from "@/shared/api/generated/QueueItem"
import {
  filterCounts,
  proposerApp,
  queueRows,
} from "@/pages/log-queue/lib/queue-rows"

const item = (id: number, state: ProposalState): QueueItem => ({
  proposal: {
    id,
    issue: "INT-290",
    date: "2026-10-09",
    startMs: 0,
    endMs: 60_000,
    minutes: 1,
    workType: null,
    text: "x",
    agent: "Codex",
    createdMs: 0,
    state,
    lastError: null,
    revision: 1,
  },
  apps: [],
})

const items = [
  item(1, { kind: "proposed" }),
  item(2, { kind: "approved", atMs: 1 }),
  item(3, { kind: "logged", atMs: 1, workItemId: "2-1" }),
  item(4, { kind: "proposed" }),
]

describe("queue rows", () => {
  it("puts a section per state and a day row before each day's proposals", () => {
    const label = (row: ReturnType<typeof queueRows>[number]) => {
      switch (row.kind) {
        case "group":
          return row.title
        case "day":
          return `${row.date} ${row.minutes}m ${row.summary?.workMs ?? "-"}`
        case "item":
          return row.item.proposal.id
      }
    }
    const summary = {
      date: "2026-10-09",
      workMs: 60_000,
      loggedMs: 0,
      proposedMs: 0,
      unloggedMs: 0,
    }

    expect(
      queueRows(items, "all", new Map([["2026-10-09", summary]])).map(label),
    ).toEqual([
      "To approve",
      "2026-10-09 2m 60000",
      1,
      4,
      "Approved · not logged yet",
      "2026-10-09 1m 60000",
      2,
      "Logged",
      "2026-10-09 1m 60000",
      3,
    ])
  })

  it("shows one group with a filter and counts every group", () => {
    expect(queueRows(items, "approved")).toHaveLength(3)
    expect(filterCounts(items)).toEqual({
      all: 4,
      proposed: 2,
      approved: 1,
      logged: 1,
      rejected: 0,
    })
  })

  it("names the agent's app for its icon", () => {
    expect(proposerApp("Claude Code").id).toBe("com.anthropic.claudefordesktop")
    expect(proposerApp("Codex").id).toBe("com.openai.codex")
  })
})
