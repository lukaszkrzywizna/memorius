import type { DayTimeline } from "@/shared/api/generated/DayTimeline"

export type DayTimelineState =
  | { kind: "loading" }
  | { kind: "error"; message: string }
  | { kind: "ready"; timeline: DayTimeline }

type DayTimelineQueryResult =
  | { status: "pending" }
  | { status: "error"; error: Error }
  | { status: "success"; data: DayTimeline }

export function dayTimelineState(
  query: DayTimelineQueryResult,
): DayTimelineState {
  switch (query.status) {
    case "pending":
      return { kind: "loading" }
    case "error":
      return { kind: "error", message: query.error.message }
    case "success":
      return { kind: "ready", timeline: query.data }
  }
}
