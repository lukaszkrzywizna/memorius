import { describe, expect, it } from "vitest"
import { needsYou } from "@/app/shell/model/needs-you"
import type { Proposal } from "@/shared/api/generated/Proposal"

const proposal = (id: number, state: Proposal["state"]): Proposal => ({
  id,
  issue: "SEL-54",
  date: "2026-10-09",
  startMs: 0,
  endMs: 60_000,
  minutes: 1,
  workType: null,
  text: "",
  agent: "Codex",
  createdMs: 0,
  state,
  lastError: null,
  revision: 1,
})

describe("needsYou", () => {
  it("counts waiting proposals only", () => {
    const needs = needsYou("2026-10-09", {
      proposals: [
        proposal(1, { kind: "proposed" }),
        proposal(2, { kind: "approved", atMs: 1 }),
      ],
      week: undefined,
    })

    expect(needs.waiting.map((p) => p.id)).toEqual([1])
    expect(needs.count).toBe(1)
  })

  it("lists today's detected issues, largest first, and not logged ones only", () => {
    const needs = needsYou("2026-10-09", {
      proposals: [],
      week: {
        days: [
          {
            date: "2026-10-08",
            activeMs: 0,
            loggedMs: 0,
            proposedMs: 0,
            unloggedMs: 0,
            hourlyActiveMs: [],
          },
          {
            date: "2026-10-09",
            activeMs: 0,
            loggedMs: 0,
            proposedMs: 0,
            unloggedMs: 65 * 60_000,
            hourlyActiveMs: [],
          },
        ],
        issues: [
          {
            issue: "SEL-21",
            cells: [{ kind: "empty" }, { kind: "logged", minutes: 30 }],
          },
          {
            issue: "INT-290",
            cells: [{ kind: "empty" }, { kind: "detected", minutes: 20 }],
          },
          {
            issue: "IS-61",
            cells: [
              { kind: "detected", minutes: 99 },
              { kind: "detected", minutes: 45 },
            ],
          },
        ],
      },
    })

    expect(needs.unlogged).toEqual([
      { issue: "IS-61", minutes: 45 },
      { issue: "INT-290", minutes: 20 },
    ])
    expect(needs.unloggedMs).toBe(65 * 60_000)
  })
})
