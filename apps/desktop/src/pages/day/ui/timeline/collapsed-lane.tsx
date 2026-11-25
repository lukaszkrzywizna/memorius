import type { SpanBox } from "@/pages/day/lib/timeline-layout"
import type { TimelineView } from "@/pages/day/lib/timeline-view"
import { blockFocus } from "@/pages/day/ui/timeline/block-style"

export type CollapsedItem = {
  key: string
  box: SpanBox
  /** What the block is and when, for the tooltip and screen readers. */
  label: string
  selected: boolean
  dimmed: boolean
  onSelect: () => void
  /** Its track when blocks run in parallel (agent sessions). */
  column?: number
}

/** Distance between parallel tracks, and the room they may take. */
const TRACK_PX = 12
const TRACKS_ROOM_PX = 72
const MARK_PX = 8

/**
 * A folded lane, as YouTrack folds a board column: each block becomes a
 * small mark at its time (longer for longer blocks) that still opens its
 * details. Parallel blocks keep separate tracks side by side.
 */
export function CollapsedLane({
  items,
  view,
  tone,
  tracks = 1,
}: {
  items: readonly CollapsedItem[]
  view: TimelineView
  /** The marks' colour, for example `bg-t3`. */
  tone: string
  /** How many parallel tracks the lane has. */
  tracks?: number
}) {
  const horizontal = view === "horizontal"
  const step = Math.min(TRACK_PX, TRACKS_ROOM_PX / tracks)
  const thickness = Math.min(MARK_PX, Math.max(3, step - 3))
  const across = (column = 0) =>
    `calc(50% + ${(column - (tracks - 1) / 2) * step}px)`
  return items.map((item) => (
    <button
      key={item.key}
      type="button"
      aria-label={item.label}
      aria-pressed={item.selected}
      title={item.label}
      onClick={item.onSelect}
      className={`absolute rounded-[2px] ${blockFocus} ${
        item.selected ? "bg-foreground" : tone
      } ${item.dimmed ? "opacity-30" : ""} ${
        horizontal ? "-translate-y-1/2" : "-translate-x-1/2"
      }`}
      style={
        horizontal
          ? {
              left: item.box.start,
              width: Math.max(MARK_PX, item.box.size),
              top: across(item.column),
              height: thickness,
            }
          : {
              top: item.box.start,
              height: Math.max(MARK_PX, item.box.size),
              left: across(item.column),
              width: thickness,
            }
      }
    />
  ))
}
