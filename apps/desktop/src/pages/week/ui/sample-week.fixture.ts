import type { WeekCell } from "@/shared/api/generated/WeekCell"
import type { WeekSummary } from "@/shared/api/generated/WeekSummary"

const minutes = (count: number) => count * 60_000
const empty: WeekCell = { kind: "empty" }
const logged = (count: number): WeekCell => ({ kind: "logged", minutes: count })
const proposed = (count: number): WeekCell => ({
  kind: "proposed",
  minutes: count,
})
const detected = (count: number): WeekCell => ({
  kind: "detected",
  minutes: count,
})

const dates = ["05", "06", "07", "08", "09", "10", "11"].map(
  (day) => `2026-10-${day}`,
)
const tracked = [460, 485, 440, 492, 479, 0, 0]
const loggedPerDay = [420, 450, 360, 210, 142, 0, 0]
const proposedPerDay = [0, 0, 40, 150, 79, 0, 0]

/** A working day's shape: busy mornings and afternoons, a dip at lunch. */
const SHAPE = [
  0, 0, 0, 0, 0, 0, 0, 0, 3, 5, 6, 4, 2, 5, 6, 5, 4, 2, 1, 0, 0, 0, 0, 0,
]
const hourly = (total: number, seed: number) => {
  const weights = SHAPE.map((w, hour) => (w ? w + ((hour + seed) % 3) : 0))
  const sum = weights.reduce((a, b) => a + b, 0)
  return weights.map((w) => Math.round(minutes((total * w) / sum)))
}

/** Week 41, shaped like the Week timesheet artboard. */
export const sampleWeek: WeekSummary = {
  days: dates.map((date, index) => ({
    date,
    activeMs: minutes(tracked[index] ?? 0),
    loggedMs: minutes(loggedPerDay[index] ?? 0),
    proposedMs: minutes(proposedPerDay[index] ?? 0),
    unloggedMs: minutes(
      (tracked[index] ?? 0) -
        (loggedPerDay[index] ?? 0) -
        (proposedPerDay[index] ?? 0),
    ),
    hourlyActiveMs: hourly(tracked[index] ?? 0, index),
  })),
  issues: [
    {
      issue: "INT-290",
      cells: [
        logged(130),
        logged(185),
        empty,
        logged(45),
        logged(112),
        empty,
        empty,
      ],
    },
    {
      issue: "INT-177",
      cells: [
        logged(200),
        logged(160),
        logged(90),
        logged(60),
        detected(74),
        empty,
        empty,
      ],
    },
    {
      issue: "SEL-54",
      cells: [
        empty,
        logged(75),
        logged(180),
        proposed(150),
        proposed(79),
        empty,
        empty,
      ],
    },
    {
      issue: "IS-61",
      cells: [
        logged(60),
        empty,
        proposed(40),
        detected(95),
        detected(104),
        empty,
        empty,
      ],
    },
    {
      issue: "SEL-21",
      cells: [
        logged(30),
        logged(30),
        logged(90),
        logged(105),
        logged(30),
        empty,
        empty,
      ],
    },
  ],
}
