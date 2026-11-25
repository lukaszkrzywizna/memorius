import { redirect } from "@tanstack/react-router"
import { todayIso } from "@/shared/lib/local-date"

/** The app opens on today's timeline. */
export function redirectToToday() {
  redirect({
    to: "/day/$date",
    params: { date: todayIso() },
    search: {},
    throw: true,
  })
}
