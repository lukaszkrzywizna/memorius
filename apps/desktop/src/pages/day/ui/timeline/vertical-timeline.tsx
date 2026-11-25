import { ChevronDownIcon } from "lucide-react"
import { useState } from "react"
import { agentPlaces } from "@/pages/day/lib/agent-lanes"
import {
  type LaneName,
  type LaneWidths,
  type ResizableLane,
  clampLane,
  laneColumns,
} from "@/pages/day/lib/lane-widths"
import {
  spanBox,
  timeMarks,
  timeOffset,
  timelineLength,
} from "@/pages/day/lib/timeline-layout"
import { AgentBlock } from "@/pages/day/ui/timeline/agent-block"
import { AppBlock } from "@/pages/day/ui/timeline/app-block"
import { CollapsedLane } from "@/pages/day/ui/timeline/collapsed-lane"
import { CommitMarks } from "@/pages/day/ui/timeline/commit-marks"
import type { TimelineViewProps } from "@/pages/day/ui/timeline/horizontal-timeline"
import { PromptTip } from "@/pages/day/ui/timeline/prompt-tip"
import {
  type PromptMark,
  type Selection,
  dimmedBy,
  laneEntries,
  promptMarks,
} from "@/pages/day/ui/timeline/timeline-model"
import { WorkBlock } from "@/pages/day/ui/timeline/work-block"
import {
  readStoredLaneWidths,
  storeLaneWidths,
} from "@/pages/day/model/lane-widths-storage"
import { formatClock } from "@/shared/lib/local-date"

const COLUMNS = "grid gap-x-2.5 max-[720px]:gap-x-1.5"
const KEY_STEP_PX = 16
const CARD_HEIGHT = 380

/** Lanes side by side, time top to bottom; scrolls down. */
export function VerticalTimeline({
  timeline,
  dayStartMs,
  scale,
  nowMs,
  selection,
  highlight,
  onSelect,
  card,
  scrollerRef,
  PromptText,
  collapsed,
  onToggleLane,
  CommitDetails,
}: TimelineViewProps) {
  const { commits } = timeline
  const [tip, setTip] = useState<PromptMark | null>(null)
  const [widths, setWidths] = useState<LaneWidths>(readStoredLaneWidths)
  const resize = (lane: ResizableLane, width: number | undefined) =>
    setWidths((current) => {
      const next = {
        ...current,
        [lane]: width === undefined ? undefined : clampLane(width),
      }
      storeLaneWidths(next)
      return next
    })
  const length = timelineLength(scale)
  const places = agentPlaces(timeline.agents)
  const parallel = Math.max(1, ...places.map((place) => place.columns))
  const columns = {
    gridTemplateColumns: laneColumns(
      widths,
      parallel,
      collapsed,
      commits.length > 0,
    ),
  }
  const dimmed = dimmedBy(timeline, highlight)
  const box = (span: { startMs: number; endMs: number }) =>
    spanBox(span, dayStartMs, scale)
  const selected = (lane: Selection["lane"], index: number) =>
    selection?.lane === lane && selection.index === index
  const folded = (lane: Selection["lane"], tone: string) => (
    <CollapsedLane
      view="vertical"
      tone={tone}
      tracks={lane === "agents" ? parallel : 1}
      items={laneEntries(timeline, lane).map((entry) => ({
        key: `${entry.span.startMs}:${entry.index}`,
        box: box(entry.span),
        label: entry.label,
        selected: selected(lane, entry.index),
        dimmed: dimmed[lane](entry.index),
        onSelect: () => onSelect({ lane, index: entry.index }),
        column: lane === "agents" ? places[entry.index]?.column : undefined,
      }))}
    />
  )
  const header = (lane: LaneName, name: string, resizable: boolean) => (
    <LaneHeader
      name={name}
      lane={lane}
      open={!collapsed.has(lane)}
      onToggle={onToggleLane}
      onResize={resizable ? resize : undefined}
    />
  )
  const anchor = (() => {
    if (!selection) return undefined
    const item =
      selection.lane === "apps"
        ? timeline.blocks[selection.index]
        : selection.lane === "agents"
          ? timeline.agents[selection.index]
          : timeline.work[selection.index]
    return item && box(item).start
  })()
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-2 px-4 pb-16 max-[720px]:px-0 max-[720px]:pb-14">
      <div
        ref={scrollerRef}
        role="region"
        aria-label="Timeline"
        className="relative min-h-0 flex-1 overflow-auto rounded-[18px] border border-(--glass-edge) bg-well pr-2.5 pb-2 [scrollbar-width:thin]"
      >
        <div
          className={`${COLUMNS} sticky top-0 z-20 bg-well pt-2 pb-2 text-t4`}
          style={columns}
        >
          <span />
          {header("apps", "Apps", true)}
          {header("agents", "Agents", true)}
          {commits.length > 0 && <span className="text-t4">Commits</span>}
          {header("work", "Work", false)}
          <span title="Your prompts">✦</span>
        </div>
        <div
          className={`relative ${COLUMNS}`}
          style={{ height: length, ...columns }}
        >
          {timeMarks(scale).map((mark) => (
            <span
              key={mark.minute}
              aria-hidden="true"
              className={`absolute right-0 left-[62px] h-px max-[720px]:left-11 ${mark.hour ? "bg-t1/10" : "bg-t1/5"}`}
              style={{ top: mark.offset }}
            />
          ))}
          <div aria-hidden="true" className="relative">
            {timeMarks(scale).map((mark) => (
              <span
                key={mark.minute}
                className="absolute right-0.5 -translate-y-1/2 text-[11px] text-t4"
                style={{ top: mark.offset }}
              >
                {mark.label}
              </span>
            ))}
          </div>
          <div aria-label="Apps" className="relative">
            {collapsed.has("apps")
              ? folded("apps", "bg-t3")
              : timeline.blocks.map((block, index) => (
                  <AppBlock
                    key={block.startMs}
                    block={block}
                    box={box(block)}
                    view="vertical"
                    selected={selected("apps", index)}
                    dimmed={dimmed.apps(index)}
                    onSelect={() => onSelect({ lane: "apps", index })}
                  />
                ))}
          </div>
          <div aria-label="Agents" className="relative">
            {collapsed.has("agents")
              ? folded("agents", "bg-t2")
              : timeline.agents.map((agent, index) => (
                  <AgentBlock
                    key={`${agent.startMs}:${agent.source}:${agent.title ?? index}`}
                    block={agent}
                    box={box(agent)}
                    view="vertical"
                    place={places[index] ?? { column: 0, columns: 1 }}
                    selected={selected("agents", index)}
                    dimmed={dimmed.agents(index)}
                    onSelect={() => onSelect({ lane: "agents", index })}
                  />
                ))}
          </div>
          {commits.length > 0 && (
            <div aria-label="Commits" className="relative">
              <CommitMarks
                timeline={timeline}
                CommitDetails={CommitDetails}
                view="vertical"
                dayStartMs={dayStartMs}
                scale={scale}
              />
            </div>
          )}
          <div aria-label="Work context" className="relative">
            {collapsed.has("work")
              ? folded("work", "bg-t4")
              : timeline.work.map((span, index) => (
                  <WorkBlock
                    key={span.startMs}
                    span={span}
                    box={box(span)}
                    view="vertical"
                    selected={selected("work", index)}
                    dimmed={dimmed.work(index)}
                    onSelect={() => onSelect({ lane: "work", index })}
                  />
                ))}
          </div>
          <div
            aria-label="Your prompts"
            className="relative border-l border-t1/10"
          >
            {promptMarks(timeline).map((prompt, index) => (
              <button
                key={prompt.atMs}
                type="button"
                tabIndex={-1}
                aria-label={prompt.label}
                onMouseEnter={() => setTip(prompt)}
                onMouseLeave={() => setTip(null)}
                className="absolute inset-x-0 flex h-[11px] cursor-default items-center pl-1"
                style={{ top: timeOffset(prompt.atMs, dayStartMs, scale) - 5 }}
              >
                <span
                  className={`h-[3px] rounded-[1px] ${tip && tip.atMs !== prompt.atMs ? "bg-t4" : "bg-t2"}`}
                  style={{ width: 10 + ((index * 5) % 14) }}
                />
              </button>
            ))}
          </div>
          {tip && (
            <PromptTip
              prompt={tip}
              PromptText={PromptText}
              style={{
                right: 44,
                top: Math.max(0, timeOffset(tip.atMs, dayStartMs, scale) - 24),
              }}
            />
          )}
          {nowMs !== null && (
            <>
              <span
                aria-hidden="true"
                className="absolute right-0 left-14 border-t border-dashed border-t1/65"
                style={{ top: timeOffset(nowMs, dayStartMs, scale) }}
              />
              <span
                className="absolute left-0 -translate-y-1/2 rounded-lg bg-t1 px-1.5 py-px text-[10px] text-primary-foreground"
                style={{ top: timeOffset(nowMs, dayStartMs, scale) }}
              >
                {formatClock(nowMs)}
              </span>
            </>
          )}
          {card && anchor !== undefined && (
            <div
              className={`absolute z-10 w-[400px] max-w-[calc(100%-70px)] max-[720px]:fixed! max-[720px]:inset-x-2.5! max-[720px]:top-auto! max-[720px]:bottom-20! max-[720px]:max-h-[60vh]! max-[720px]:w-auto! max-[720px]:max-w-none! max-[720px]:overflow-y-auto ${
                selection?.lane === "apps" ? "left-[calc(62px+33%)]" : "left-16"
              }`}
              style={{
                top: Math.max(0, Math.min(anchor - 6, length - CARD_HEIGHT)),
              }}
            >
              {card}
            </div>
          )}
        </div>
      </div>
      <p className="absolute bottom-6 left-6.5 m-0 text-xs text-t5 max-[720px]:hidden">
        Scroll down · click a block for details
      </p>
    </div>
  )
}

/**
 * A lane's name that folds the lane, with a handle on its right edge: drag
 * it, or use the arrow keys, to set the lane's width; a double click gives
 * the width back to the automatic layout.
 */
function LaneHeader({
  name,
  lane,
  open,
  onToggle,
  onResize,
}: {
  name: string
  lane: LaneName
  open: boolean
  onToggle: (lane: LaneName) => void
  /** Missing for a lane that only folds. */
  onResize:
    | ((lane: ResizableLane, width: number | undefined) => void)
    | undefined
}) {
  return (
    <span className="relative min-w-0">
      <button
        type="button"
        aria-expanded={open}
        aria-label={open ? `Fold ${name}` : `Unfold ${name}`}
        title={open ? `Fold ${name}` : `Unfold ${name}`}
        onClick={() => onToggle(lane)}
        className="inline-flex max-w-full items-center gap-1 rounded-md text-t4 outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
      >
        <ChevronDownIcon
          aria-hidden="true"
          className={`size-3 shrink-0 transition-transform ${open ? "" : "-rotate-90"}`}
        />
        <span className="truncate">{name}</span>
      </button>
      {open && onResize && lane !== "work" && (
        <span
          role="separator"
          aria-orientation="vertical"
          aria-label={`Resize ${name}`}
          tabIndex={0}
          title="Drag to resize · double-click to reset"
          className="absolute inset-y-0 -right-[11px] w-3 cursor-col-resize rounded outline-none after:absolute after:inset-y-0.5 after:left-1/2 after:w-px after:bg-t1/15 hover:after:bg-t1/40 focus-visible:ring-2 focus-visible:ring-ring"
          onDoubleClick={() => onResize(lane, undefined)}
          onKeyDown={(event) => {
            const width =
              event.currentTarget.parentElement?.getBoundingClientRect().width
            if (width === undefined) return
            if (event.key === "ArrowLeft") onResize(lane, width - KEY_STEP_PX)
            if (event.key === "ArrowRight") onResize(lane, width + KEY_STEP_PX)
          }}
          onPointerDown={(event) => {
            const handle = event.currentTarget
            const start = handle.parentElement?.getBoundingClientRect().width
            if (start === undefined) return
            const startX = event.clientX
            handle.setPointerCapture(event.pointerId)
            const move = (moved: PointerEvent) =>
              onResize(lane, start + moved.clientX - startX)
            const stop = () => {
              handle.removeEventListener("pointermove", move)
              handle.removeEventListener("pointerup", stop)
            }
            handle.addEventListener("pointermove", move)
            handle.addEventListener("pointerup", stop)
          }}
        />
      )}
    </span>
  )
}
