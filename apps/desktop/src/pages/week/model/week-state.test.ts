import { describe, expect, it } from "vitest"
import { weekState } from "@/pages/week/model/week-state"

describe("weekState", () => {
  it("maps each query status to one page state", () => {
    const summary = { days: [], issues: [] }

    expect(weekState({ status: "pending" })).toEqual({ kind: "loading" })
    expect(weekState({ status: "error", error: new Error("locked") })).toEqual({
      kind: "error",
      message: "locked",
    })
    expect(weekState({ status: "success", data: summary })).toEqual({
      kind: "ready",
      summary,
    })
  })
})
