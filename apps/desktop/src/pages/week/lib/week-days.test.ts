import { describe, expect, it } from "vitest"
import type { WeekDay } from "@/shared/api/generated/WeekDay"
import {
  isoWeekNumber,
  sparkline,
  visibleDays,
  weekRange,
  weekTitle,
  weeksAgo,
} from "@/pages/week/lib/week-days"

const day = (date: string, activeMs = 0): WeekDay => ({
  date,
  activeMs,
  loggedMs: 0,
  proposedMs: 0,
  unloggedMs: 0,
  hourlyActiveMs: [],
})

const week = ["05", "06", "07", "08", "09", "10", "11"].map((d) =>
  day(`2026-10-${d}`),
)

describe("week days", () => {
  it("numbers ISO weeks across year ends", () => {
    expect(isoWeekNumber("2026-10-09")).toBe(41)
    expect(isoWeekNumber("2027-01-01")).toBe(53)
    expect(isoWeekNumber("2026-01-01")).toBe(1)
  })

  it("names the week and its dates, also across months", () => {
    expect(weekTitle(week)).toBe("Week 41")
    expect(weekRange(week)).toBe("5 – 11 October 2026")
    expect(weekRange([day("2026-09-28"), day("2026-10-04")])).toBe(
      "28 Sept – 4 October 2026",
    )
  })

  it("says how long ago the week was", () => {
    expect(weeksAgo(week, "2026-10-09")).toBe("this week")
    expect(weeksAgo(week, "2026-10-14")).toBe("last week")
    expect(weeksAgo(week, "2026-10-28")).toBe("3 weeks ago")
  })

  it("draws the working hours as a sparkline from 07:00", () => {
    const hours = Array.from({ length: 24 }, (_, hour) =>
      hour === 9 ? 3_600_000 : 0,
    )
    const points = sparkline(hours).split(" ")
    expect(points).toHaveLength(14)
    expect(points[2]).toBe("15.4,4.0")
    expect(points[0]).toBe("0.0,24.0")
  })

  it("shows weekend days only when they have activity", () => {
    const withSaturday = week.map((d, index) =>
      index === 5 ? day(d.date, 60_000) : d,
    )

    expect(visibleDays(week)).toHaveLength(5)
    expect(visibleDays(withSaturday).map(({ index }) => index)).toEqual([
      0, 1, 2, 3, 4, 5,
    ])
  })
})
