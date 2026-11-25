const MINUTE_MS = 60_000

/** `7h 59m`, `47m`, `2h`; anything under a minute is `<1m`. */
export function formatDuration(ms: number) {
  const minutes = Math.floor(ms / MINUTE_MS)
  if (minutes < 1) {
    return "<1m"
  }
  const hours = Math.floor(minutes / 60)
  const rest = minutes % 60
  if (hours === 0) {
    return `${rest}m`
  }
  return rest === 0 ? `${hours}h` : `${hours}h ${rest}m`
}
