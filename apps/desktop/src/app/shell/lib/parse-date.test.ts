import { describe, expect, it } from "vitest"
import { parseDay } from "@/app/shell/lib/parse-date"

// Friday 9 October 2026.
const today = "2026-10-09"

describe("parseDay", () => {
  it("reads relative days and weekdays", () => {
    expect(parseDay("today", today)).toBe("2026-10-09")
    expect(parseDay("yest", today)).toBe("2026-10-08")
    expect(parseDay("tuesday", today)).toBe("2026-10-06")
    expect(parseDay("fri", today)).toBe("2026-10-09")
    expect(parseDay("last monday", today)).toBe("2026-09-28")
  })

  it("reads day and month in several forms", () => {
    expect(parseDay("28 sep", today)).toBe("2026-09-28")
    expect(parseDay("3.10", today)).toBe("2026-10-03")
    expect(parseDay("2026-03-29", today)).toBe("2026-03-29")
  })

  it("ignores text that is not a date", () => {
    expect(parseDay("is-61", today)).toBeUndefined()
    expect(parseDay("31 feb", today)).toBeUndefined()
    expect(parseDay("t", today)).toBeUndefined()
  })
})
