import type { Block } from "@/shared/api/generated/Block"
import { AppIcon, windowTitle } from "@/entities/activity"
import type { SpanBox } from "@/pages/day/lib/timeline-layout"
import type { TimelineView } from "@/pages/day/lib/timeline-view"
import {
  axisStyle,
  blockFocus,
  blockSurface,
} from "@/pages/day/ui/timeline/block-style"
import { formatDuration } from "@/shared/lib/duration"
import { formatClock } from "@/shared/lib/local-date"

type AppBlockProps = {
  block: Block
  box: SpanBox
  view: TimelineView
  selected: boolean
  dimmed: boolean
  onSelect: () => void
}

/** An app block; away and locked time show as hatched gaps. */
export function AppBlock({
  block,
  box,
  view,
  selected,
  dimmed,
  onSelect,
}: AppBlockProps) {
  const range = `${formatClock(block.startMs)}–${formatClock(block.endMs)}`
  const duration = formatDuration(block.endMs - block.startMs)
  const across = view === "horizontal" ? "inset-y-0" : "inset-x-0"
  switch (block.kind) {
    case "away":
    case "locked":
      return (
        <div
          className={`absolute ${across} flex items-start justify-between gap-2 overflow-hidden rounded-[10px] border border-(--glass-edge) bg-background bg-[repeating-linear-gradient(135deg,color-mix(in_srgb,var(--t1)_7%,transparent)_0_1px,transparent_1px_6px)] px-2.5 py-1.5 text-t4`}
          style={axisStyle(box, view)}
        >
          {box.size >= 40 && (
            <>
              <span>{block.kind === "away" ? "Away" : "Locked"}</span>
              <span>{duration}</span>
            </>
          )}
        </div>
      )
    case "apps": {
      const title = windowTitle(block.title)
      const horizontal = view === "horizontal"
      const showIcon = box.size >= (horizontal ? 30 : 22)
      const showTitle = box.size >= (horizontal ? 64 : 22)
      const showDuration = box.size >= (horizontal ? 40 : 22)
      return (
        <button
          type="button"
          aria-label={`${title}, ${block.dominant.name}, ${range}`}
          aria-pressed={selected}
          onClick={onSelect}
          className={`absolute ${across} flex overflow-hidden rounded-[10px] border text-left ${blockSurface(selected)} ${blockFocus} ${
            horizontal ? "flex-col gap-1.5 p-2" : "gap-2 px-2.5 py-1"
          } ${dimmed ? "opacity-40" : ""}`}
          style={axisStyle(box, view)}
        >
          {showIcon && (
            <AppIcon app={block.dominant} size={horizontal ? 22 : 20} />
          )}
          <span className="flex min-w-0 flex-1 flex-col gap-0.5">
            {showTitle && (
              <span className="truncate text-t1" title={title}>
                {title}
              </span>
            )}
            {!horizontal && box.size >= 52 && (
              <span className="truncate text-xs text-t4">
                {block.dominant.name} · {range}
              </span>
            )}
          </span>
          {showDuration && (
            <span
              className={`whitespace-nowrap ${horizontal ? "text-xs text-t4" : "text-t1"}`}
            >
              {duration}
            </span>
          )}
        </button>
      )
    }
  }
}
