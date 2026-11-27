import { describe, expect, it } from "vitest"
import { dayTimelineState } from "@/pages/day/model/day-timeline-state"

const timeline = {
  fromMs: 0,
  toMs: 1,
  blocks: [],
  agents: [],
  work: [],
  commits: [],
  logged: { blocks: [], gaps: [] },
  totals: {
    activeMs: 0,
    awayMs: 0,
    agentMs: 0,
    split: { workMs: 0, agentMs: 0, uncountedMs: 0 },
  },
}

describe("dayTimelineState", () => {
  it("maps each query status to one page state", () => {
    expect(dayTimelineState({ status: "pending" })).toEqual({ kind: "loading" })
    expect(
      dayTimelineState({ status: "error", error: new Error("locked") }),
    ).toEqual({
      kind: "error",
      message: "locked",
    })
    expect(dayTimelineState({ status: "success", data: timeline })).toEqual({
      kind: "ready",
      timeline,
    })
  })
})
