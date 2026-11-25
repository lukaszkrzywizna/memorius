import { useEffect, useState } from "react"

const MINUTE_MS = 60_000

/** The current time, refreshed every minute for the "now" line. */
export function useNow() {
  const [now, setNow] = useState(Date.now)
  useEffect(function tickEveryMinute() {
    const timer = setInterval(() => setNow(Date.now()), MINUTE_MS)
    return () => clearInterval(timer)
  }, [])
  return now
}
