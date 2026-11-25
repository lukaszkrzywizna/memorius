import { describe, expect, it } from "vitest"
import {
  SLOT_PX,
  spanBox,
  timeMarks,
  timelineScale,
} from "@/pages/day/lib/timeline-layout"

const DAY = 0
const at = (hour: number, minute = 0) => (hour * 60 + minute) * 60_000

describe("timeline layout", () => {
  it("shows 08:00–18:00 when the activity fits inside", () => {
    expect(
      timelineScale(
        [{ startMs: at(9), endMs: at(17, 30) }],
        DAY,
        15,
        "vertical",
      ),
    ).toMatchObject({ startHour: 8, endHour: 18 })
  })

  it("widens to whole hours around early and late activity", () => {
    expect(
      timelineScale(
        [{ startMs: at(6, 45), endMs: at(21, 10) }],
        DAY,
        15,
        "horizontal",
      ),
    ).toMatchObject({ startHour: 6, endHour: 22 })
  })

  it("gives every slot the same length at every zoom", () => {
    for (const view of ["horizontal", "vertical"] as const) {
      for (const zoom of [1, 15, 60] as const) {
        const scale = timelineScale([], DAY, zoom, view)
        const slot = spanBox({ startMs: at(9), endMs: at(9, zoom) }, DAY, scale)
        expect(slot.size).toBe(SLOT_PX[view] - 3)
      }
    }
  })

  it("places a span from the first visible hour", () => {
    const scale = timelineScale([], DAY, 15, "vertical")
    expect(spanBox({ startMs: at(9, 30), endMs: at(10) }, DAY, scale)).toEqual({
      start: 6 * SLOT_PX.vertical,
      size: 2 * SLOT_PX.vertical - 3,
    })
  })

  it("labels whole hours from 15-minute slots and closer below", () => {
    const step = (zoom: 1 | 5 | 15) => {
      const marks = timeMarks(timelineScale([], DAY, zoom, "horizontal"))
      return (marks[1]?.minute ?? 0) - (marks[0]?.minute ?? 0)
    }
    expect([step(15), step(5), step(1)]).toEqual([60, 30, 15])
  })
})
