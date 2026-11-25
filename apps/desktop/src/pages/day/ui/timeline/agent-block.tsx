import type { AgentBlock as AgentBlockData } from "@/shared/api/generated/AgentBlock"
import { AppIcon } from "@/entities/activity"
import { type AgentPlace, agentApp } from "@/pages/day/lib/agent-lanes"
import type { SpanBox } from "@/pages/day/lib/timeline-layout"
import type { TimelineView } from "@/pages/day/lib/timeline-view"
import {
  axisStyle,
  blockFocus,
  blockSurface,
} from "@/pages/day/ui/timeline/block-style"
import { formatClock } from "@/shared/lib/local-date"

type AgentBlockProps = {
  block: AgentBlockData
  box: SpanBox
  view: TimelineView
  /** Parallel sessions share the lane: rows sideways, columns downwards. */
  place: AgentPlace
  selected: boolean
  dimmed: boolean
  onSelect: () => void
}

export function AgentBlock({
  block,
  box,
  view,
  place,
  selected,
  dimmed,
  onSelect,
}: AgentBlockProps) {
  const app = agentApp(block.source)
  const title = block.title ?? block.branch ?? app.name
  const horizontal = view === "horizontal"
  const note = block.private
    ? "Private"
    : block.issue
      ? undefined
      : "No ticket found"
  const share = 100 / place.columns
  const cross = horizontal
    ? {
        top: `calc(${place.column * share}% + 1px)`,
        height: `calc(${share}% - 4px)`,
      }
    : {
        left: `calc(${place.column * share}% + 2px)`,
        width: `calc(${share}% - 4px)`,
      }
  return (
    <button
      type="button"
      aria-label={`${app.name}, ${title}${block.issue ? `, ${block.issue}` : ""}, ${formatClock(block.startMs)}–${formatClock(block.endMs)}${note ? `, ${note.toLowerCase()}` : ""}`}
      title={note}
      aria-pressed={selected}
      onClick={onSelect}
      className={`absolute flex overflow-hidden border text-left whitespace-nowrap ${blockFocus} ${
        block.issue && !block.private
          ? blockSurface(selected)
          : `border-dashed ${selected ? "border-ring/60 bg-selected" : "border-t5"}`
      } ${
        horizontal
          ? "items-center gap-2 rounded-full py-0 pr-3 pl-1.5"
          : "flex-col gap-0.5 rounded-[10px] px-2 py-1"
      } ${dimmed || (block.private && !selected) ? "opacity-40" : ""}`}
      style={{ ...axisStyle(box, view), ...cross }}
    >
      <span className="flex min-w-0 items-center gap-1.5">
        <AppIcon app={app} size={horizontal ? 24 : 18} />
        {(horizontal || box.size >= 22) && (
          <span className="truncate text-t1">{block.issue ?? title}</span>
        )}
      </span>
      {block.issue && (horizontal || box.size >= 46) && (
        <span className="min-w-0 truncate text-xs text-t4">{title}</span>
      )}
    </button>
  )
}
