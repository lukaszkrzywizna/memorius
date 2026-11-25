import { addDays, isValidIsoDate } from "@/shared/lib/local-date"

const WEEKDAYS = [
  "sunday",
  "monday",
  "tuesday",
  "wednesday",
  "thursday",
  "friday",
  "saturday",
]
const MONTHS = [
  "january",
  "february",
  "march",
  "april",
  "may",
  "june",
  "july",
  "august",
  "september",
  "october",
  "november",
  "december",
]

const iso = (year: number, month: number, day: number) =>
  `${year}-${String(month + 1).padStart(2, "0")}-${String(day).padStart(2, "0")}`

/**
 * A day the user typed: "today", "yesterday", "tuesday", "last monday",
 * "28 sep", "3.10" or "2026-10-03"; calendar days in local time, so DST
 * never shifts them. Weekdays mean the latest one up to today.
 */
export function parseDay(text: string, today: string): string | undefined {
  const query = text.trim().toLowerCase()
  if (query.length < 2) return undefined
  if (isValidIsoDate(query)) return query
  if ("today".startsWith(query)) return today
  if ("yesterday".startsWith(query)) return addDays(today, -1)
  const last = /^last\s+([a-z]+)$/.exec(query)
  const name = last ? last[1] : query
  const weekday = WEEKDAYS.findIndex(
    (day) => name !== undefined && name.length >= 3 && day.startsWith(name),
  )
  if (weekday >= 0) {
    const now = new Date(`${today}T12:00:00`)
    const back = (now.getDay() - weekday + 7) % 7
    return addDays(today, -(back + (last ? 7 : 0)))
  }
  const match = /^(\d{1,2})(?:[\s./-]+([a-z]+|\d{1,2}))?$/.exec(query)
  if (!match) return undefined
  const day = Number(match[1])
  const todayDate = new Date(`${today}T12:00:00`)
  const monthText = match[2]
  const month =
    monthText === undefined
      ? todayDate.getMonth()
      : /^\d+$/.test(monthText)
        ? Number(monthText) - 1
        : MONTHS.findIndex(
            (candidate) =>
              monthText.length >= 3 && candidate.startsWith(monthText),
          )
  if (month < 0 || month > 11) return undefined
  const result = iso(todayDate.getFullYear(), month, day)
  return isValidIsoDate(result) ? result : undefined
}
