/** The timeline runs left to right or top to bottom; both stay available. */
export const TIMELINE_VIEWS = ["horizontal", "vertical"] as const

export type TimelineView = (typeof TIMELINE_VIEWS)[number]

export const DEFAULT_VIEW: TimelineView = "horizontal"

export function parseTimelineView(
  value: string | null | undefined,
): TimelineView | undefined {
  return TIMELINE_VIEWS.find((view) => view === value)
}
