import type { WeekDay } from "@/shared/api/generated/WeekDay"
import { parseIsoDate } from "@/shared/lib/local-date"

/** ISO 8601 week number: weeks start on Monday, week 1 holds 4 January. */
export function isoWeekNumber(iso: string) {
  const date = parseIsoDate(iso)
  const thursday = new Date(date)
  thursday.setDate(date.getDate() + 3 - ((date.getDay() + 6) % 7))
  const firstThursday = new Date(thursday.getFullYear(), 0, 4)
  firstThursday.setDate(
    firstThursday.getDate() + 3 - ((firstThursday.getDay() + 6) % 7),
  )
  return (
    1 +
    Math.round(
      (thursday.getTime() - firstThursday.getTime()) / (7 * 86_400_000),
    )
  )
}

const dayMonth = new Intl.DateTimeFormat("en-GB", {
  day: "numeric",
  month: "short",
})
const dayMonthYear = new Intl.DateTimeFormat("en-GB", {
  day: "numeric",
  month: "long",
  year: "numeric",
})

/** `Week 41` */
export function weekTitle(days: readonly WeekDay[]) {
  // The middle day names the ISO week whether weeks start Monday or Sunday.
  const middle = days[3]
  return middle ? `Week ${isoWeekNumber(middle.date)}` : "Week"
}

/** `5 – 11 October 2026`, or `28 Sep – 4 October 2026` across months. */
export function weekRange(days: readonly WeekDay[]) {
  const first = days[0]
  const last = days.at(-1)
  if (!first || !last) return ""
  const from = parseIsoDate(first.date)
  const to = parseIsoDate(last.date)
  const start =
    from.getMonth() === to.getMonth()
      ? String(from.getDate())
      : dayMonth.format(from)
  return `${start} – ${dayMonthYear.format(to)}`
}

/** `this week`, `last week` or `3 weeks ago`, counted from Mondays. */
export function weeksAgo(days: readonly WeekDay[], today: string) {
  const first = days[0]
  if (!first) return ""
  const monday = parseIsoDate(today)
  monday.setDate(monday.getDate() - ((monday.getDay() + 6) % 7))
  const weeks = Math.round(
    (monday.getTime() - parseIsoDate(first.date).getTime()) / (7 * 86_400_000),
  )
  return weeks <= 0
    ? "this week"
    : weeks === 1
      ? "last week"
      : `${weeks} weeks ago`
}

/** Monday to Friday, plus weekend days that have activity. */
export function visibleDays(days: readonly WeekDay[]) {
  return days
    .map((day, index) => ({ day, index }))
    .filter(
      ({ day, index }) => index < 5 || day.activeMs > 0 || day.loggedMs > 0,
    )
}

const weekday = new Intl.DateTimeFormat("en-GB", {
  weekday: "short",
  day: "numeric",
})

/** `Mon 5` */
export function shortDay(iso: string) {
  return weekday.format(parseIsoDate(iso))
}

const longDay = new Intl.DateTimeFormat("en-GB", { weekday: "long" })
const shortDate = new Intl.DateTimeFormat("en-GB", {
  day: "numeric",
  month: "short",
})

/** `Monday` and `5 Oct`, for the day cards. */
export function dayCardLabels(iso: string) {
  const date = parseIsoDate(iso)
  return { name: longDay.format(date), date: shortDate.format(date) }
}

/** Points for a 100×26 sparkline of the working hours (07:00–20:00). */
export function sparkline(hourlyActiveMs: readonly number[]) {
  const hours = hourlyActiveMs.slice(7, 21)
  const max = Math.max(1, ...hours)
  const last = Math.max(1, hours.length - 1)
  return hours
    .map(
      (ms, index) =>
        `${((index / last) * 100).toFixed(1)},${(24 - (ms / max) * 20).toFixed(1)}`,
    )
    .join(" ")
}
