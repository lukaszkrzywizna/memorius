/** Local calendar dates as `YYYY-MM-DD`, the format the day route and the Rust commands use. */

const pad = (value: number) => String(value).padStart(2, "0")

function isoDate(date: Date) {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`
}

export function todayIso(now = new Date()) {
  return isoDate(now)
}

/** Local midnight of an ISO date. Invalid input yields an invalid Date. */
export function parseIsoDate(iso: string) {
  const [year, month, day] = iso.split("-").map(Number)
  return new Date(
    year ?? Number.NaN,
    (month ?? Number.NaN) - 1,
    day ?? Number.NaN,
  )
}

export function isValidIsoDate(iso: string) {
  return /^\d{4}-\d{2}-\d{2}$/.test(iso) && isoDate(parseIsoDate(iso)) === iso
}

export function addDays(iso: string, days: number) {
  const date = parseIsoDate(iso)
  date.setDate(date.getDate() + days)
  return isoDate(date)
}

const longDate = new Intl.DateTimeFormat("en-GB", {
  weekday: "long",
  day: "numeric",
  month: "long",
  year: "numeric",
})

export function formatLongDate(iso: string) {
  return longDate.format(parseIsoDate(iso))
}

const plainDate = new Intl.DateTimeFormat("en-GB", {
  day: "numeric",
  month: "long",
  year: "numeric",
})

/** "9 October 2026": the date without the weekday. */
export function formatPlainDate(iso: string) {
  return plainDate.format(parseIsoDate(iso))
}

// The system's 12- or 24-hour clock; day and month names stay English.
const systemCycle = new Intl.DateTimeFormat(navigator.language, {
  hour: "numeric",
}).resolvedOptions().hourCycle
const twelveHour = systemCycle === "h11" || systemCycle === "h12"
const clock = new Intl.DateTimeFormat("en-GB", {
  hour: twelveHour ? "numeric" : "2-digit",
  minute: "2-digit",
  hourCycle: twelveHour ? "h12" : "h23",
})

/** `13:47`, or `1:47 pm` where the system uses a 12-hour clock. */
export function formatClock(ms: number) {
  return clock.format(new Date(ms))
}

/** The clock label of a minute of the day (0–1440). */
export function formatMinuteOfDay(minute: number) {
  return clock.format(new Date(2000, 0, 1, 0, minute))
}

const shortDate = new Intl.DateTimeFormat("en-GB", {
  weekday: "short",
  day: "numeric",
  month: "short",
})

/** `Fri 9 Oct` for a moment in local time. */
export function formatShortDate(ms: number) {
  return shortDate.format(new Date(ms))
}
