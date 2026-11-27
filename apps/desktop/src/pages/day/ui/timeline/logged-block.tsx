import type { Gap } from "@/shared/api/generated/Gap"
import type { LoggedBlock as Logged } from "@/shared/api/generated/LoggedBlock"
import type { SpanBox } from "@/pages/day/lib/timeline-layout"
import type { TimelineView } from "@/pages/day/lib/timeline-view"
import {
  axisStyle,
  blockFocus,
  blockSurface,
} from "@/pages/day/ui/timeline/block-style"
import { formatDuration } from "@/shared/lib/duration"
import { formatClock } from "@/shared/lib/local-date"

type LoggedBlockProps = {
  block: Logged
  box: SpanBox
  view: TimelineView
  selected: boolean
  dimmed: boolean
  onSelect: () => void
}

/** What a block in the YouTrack lane is, in a few words. */
export function loggedNote(block: Logged) {
  switch (block.state) {
    case "logged":
      return block.estimated ? "logged · time estimated" : "logged"
    case "approved":
      return "approved · not logged yet"
    case "proposed":
      return "proposed · waiting for you"
  }
}

/**
 * Logged time is solid glass; a guessed place has a dashed edge; a proposal
 * on its way is only an outline.
 */
export function LoggedBlock({
  block,
  box,
  view,
  selected,
  dimmed,
  onSelect,
}: LoggedBlockProps) {
  const horizontal = view === "horizontal"
  const across = horizontal ? "inset-y-0" : "inset-x-0"
  const surface = selected
    ? blockSurface(true)
    : block.state !== "logged"
      ? "border-t4 bg-transparent"
      : block.estimated
        ? "glass border-dashed"
        : "glass"
  return (
    <button
      type="button"
      aria-label={`${block.issue}, ${formatDuration(block.minutes * 60_000)}, ${loggedNote(block)}, ${formatClock(block.startMs)}–${formatClock(block.endMs)}`}
      aria-pressed={selected}
      onClick={onSelect}
      className={`absolute ${across} flex flex-col justify-center gap-0.5 overflow-hidden rounded-[10px] border text-left whitespace-nowrap ${surface} ${blockFocus} ${
        horizontal ? "px-2.5" : "px-2.5 py-1"
      } ${dimmed ? "opacity-50" : ""}`}
      style={axisStyle(box, view)}
    >
      {(horizontal || box.size >= 18) && (
        <span className="text-t1">{block.issue}</span>
      )}
      {(horizontal || box.size >= 44) && (
        <span className="truncate text-[11px] text-t4">
          {loggedNote(block)}
        </span>
      )}
    </button>
  )
}

/** Work that nothing logged or proposed covers. */
export function GapMark({
  gap,
  box,
  view,
}: {
  gap: Gap
  box: SpanBox
  view: TimelineView
}) {
  const label = `Not logged, ${formatClock(gap.startMs)}–${formatClock(gap.endMs)}`
  return (
    <span
      role="img"
      aria-label={label}
      title={label}
      className={`absolute ${view === "horizontal" ? "inset-y-0" : "inset-x-0"} rounded-[10px] border border-dashed border-t4/60 bg-[repeating-linear-gradient(135deg,var(--color-t5)_0_1px,transparent_1px_7px)] opacity-70`}
      style={axisStyle(box, view)}
    />
  )
}
