import type { WeekSummary } from "@/shared/api/generated/WeekSummary"

export type WeekState =
  | { kind: "loading" }
  | { kind: "error"; message: string }
  | { kind: "ready"; summary: WeekSummary }

type WeekQueryResult =
  | { status: "pending" }
  | { status: "error"; error: Error }
  | { status: "success"; data: WeekSummary }

export function weekState(query: WeekQueryResult): WeekState {
  switch (query.status) {
    case "pending":
      return { kind: "loading" }
    case "error":
      return { kind: "error", message: query.error.message }
    case "success":
      return { kind: "ready", summary: query.data }
  }
}
