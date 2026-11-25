import { describe, expect, it } from "vitest"
import {
  addDays,
  formatLongDate,
  isValidIsoDate,
  todayIso,
} from "@/shared/lib/local-date"

describe("local dates", () => {
  it("formats today from a local Date", () => {
    expect(todayIso(new Date(2026, 9, 9, 23, 30))).toBe("2026-10-09")
  })

  it("moves across month and year ends", () => {
    expect(addDays("2026-10-31", 1)).toBe("2026-11-01")
    expect(addDays("2027-01-01", -1)).toBe("2026-12-31")
  })

  it("rejects dates that do not exist", () => {
    expect(isValidIsoDate("2026-10-09")).toBe(true)
    expect(isValidIsoDate("2026-02-30")).toBe(false)
    expect(isValidIsoDate("09.10.2026")).toBe(false)
  })

  it("writes the long English date", () => {
    expect(formatLongDate("2026-10-09")).toMatch(/^Friday,? 9 October 2026$/)
  })
})
