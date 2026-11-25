import type { SpanBox } from "@/pages/day/lib/timeline-layout"
import type { TimelineView } from "@/pages/day/lib/timeline-view"

/** Absolute placement along the time axis; the lane sets the other axis. */
export function axisStyle(box: SpanBox, view: TimelineView) {
  return view === "horizontal"
    ? { left: box.start, width: box.size }
    : { top: box.start, height: box.size }
}

/** Glass, selected frost, or a dimmed block outside a highlight. */
export function blockSurface(selected: boolean) {
  return selected
    ? "bg-selected border-ring/40 shadow-(--shadow-frost)"
    : "glass"
}

export const blockFocus =
  "cursor-pointer outline-none transition-colors hover:border-ring/60 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-foreground"
