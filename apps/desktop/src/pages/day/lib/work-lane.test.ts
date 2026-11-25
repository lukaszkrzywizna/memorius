import { describe, expect, it } from "vitest"
import type { WorkSpan } from "@/shared/api/generated/WorkSpan"
import { workLabel, workSource } from "@/pages/day/lib/work-lane"

const span = (evidence: WorkSpan["evidence"]): WorkSpan => ({
  startMs: 0,
  endMs: 900_000,
  issue: "INT-290",
  confidence: "high",
  support: 100,
  candidates: [],
  split: { workMs: 900_000, agentMs: 0, uncountedMs: 0 },
  evidence,
})

describe("work lane text", () => {
  it("names a span by its first evidence", () => {
    const work = span([
      {
        kind: "session",
        source: "codex",
        title: "Order sync",
        branch: "codex/int-290",
        prompts: 2,
      },
      { kind: "commit", subject: "[INT-290] Skip unchanged" },
    ])

    expect(workLabel(work)).toBe("Order sync")
    expect(workSource(work)).toBe("Codex session")
  })

  it("says nothing when a span has no evidence", () => {
    expect(workLabel(span([]))).toBe("")
  })
})
