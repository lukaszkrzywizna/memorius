import { describe, expect, it } from "vitest"
import type { AgentBlock } from "@/shared/api/generated/AgentBlock"
import { agentPlaces } from "@/pages/day/lib/agent-lanes"

const MIN = 60_000
const block = (start: number, end: number): AgentBlock => ({
  startMs: start * MIN,
  endMs: end * MIN,
  source: "claudeCode",
  title: null,
  branch: null,
  prompts: 0,
  promptTimesMs: [],
  issue: null,
  commits: [],
  split: { workMs: 0, agentMs: 0, uncountedMs: 0 },
  sessions: [],
  private: false,
})

describe("agentPlaces", () => {
  it("puts parallel sessions side by side and lone ones full width", () => {
    expect(
      agentPlaces([block(0, 30), block(0, 15), block(15, 45), block(60, 75)]),
    ).toEqual([
      { column: 0, columns: 2 },
      { column: 1, columns: 2 },
      { column: 1, columns: 2 },
      { column: 0, columns: 1 },
    ])
  })
})
