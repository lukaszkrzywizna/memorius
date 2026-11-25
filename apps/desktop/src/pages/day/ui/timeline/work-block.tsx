import type { WorkSpan } from "@/shared/api/generated/WorkSpan"
import { workLabel, workSource } from "@/pages/day/lib/work-lane"
import type { SpanBox } from "@/pages/day/lib/timeline-layout"
import type { TimelineView } from "@/pages/day/lib/timeline-view"
import {
  axisStyle,
  blockFocus,
  blockSurface,
} from "@/pages/day/ui/timeline/block-style"
import { formatClock } from "@/shared/lib/local-date"

type WorkBlockProps = {
  span: WorkSpan
  box: SpanBox
  view: TimelineView
  selected: boolean
  dimmed: boolean
  onSelect: () => void
}

/** What a stretch was about: an issue or a candidate (?). */
export function WorkBlock({
  span,
  box,
  view,
  selected,
  dimmed,
  onSelect,
}: WorkBlockProps) {
  const candidate = span.confidence === "medium"
  const issue = candidate ? `${span.issue}?` : span.issue
  const horizontal = view === "horizontal"
  const across = horizontal ? "inset-y-0" : "inset-x-0"
  const surface = selected
    ? blockSurface(true)
    : candidate
      ? "border-dashed border-t5"
      : "glass"
  return (
    <button
      type="button"
      aria-label={`${issue}, ${formatClock(span.startMs)}–${formatClock(span.endMs)}`}
      aria-pressed={selected}
      onClick={onSelect}
      className={`absolute ${across} flex flex-col justify-center gap-0.5 overflow-hidden rounded-[10px] border text-left whitespace-nowrap ${surface} ${blockFocus} ${
        horizontal ? "px-2.5" : "px-2.5 py-1"
      } ${dimmed ? "opacity-50" : ""}`}
      style={axisStyle(box, view)}
    >
      {(horizontal || box.size >= 18) && (
        <span className="text-t1">{issue}</span>
      )}
      {(horizontal || box.size >= 44) && (
        <span className="truncate text-[11px] text-t4">
          {candidate
            ? `candidate · ${workLabel(span)}`
            : `from ${workSource(span)}`}
        </span>
      )}
    </button>
  )
}
