import { ChevronDownIcon } from "lucide-react"
import { type ReactNode, type RefObject, useState } from "react"
import type { DayTimeline } from "@/shared/api/generated/DayTimeline"
import { agentPlaces } from "@/pages/day/lib/agent-lanes"
import {
  type TimelineScale,
  spanBox,
  timeMarks,
  timeOffset,
  timelineLength,
} from "@/pages/day/lib/timeline-layout"
import { AgentBlock } from "@/pages/day/ui/timeline/agent-block"
import type { LaneName } from "@/pages/day/lib/lane-widths"
import { AppBlock } from "@/pages/day/ui/timeline/app-block"
import { CollapsedLane } from "@/pages/day/ui/timeline/collapsed-lane"
import type { CommitDetailsView } from "@/pages/day/ui/timeline/commit-card"
import { CommitMarks } from "@/pages/day/ui/timeline/commit-marks"
import {
  type PromptTextView,
  PromptTip,
} from "@/pages/day/ui/timeline/prompt-tip"
import {
  type PromptMark,
  type Selection,
  dimmedBy,
  laneEntries,
  promptMarks,
} from "@/pages/day/ui/timeline/timeline-model"
import { WorkBlock } from "@/pages/day/ui/timeline/work-block"
import { formatClock } from "@/shared/lib/local-date"
import {
  LABEL_PX,
  horizontalLanes,
} from "@/pages/day/ui/timeline/horizontal-geometry"

const CARD_WIDTH = 460

export type TimelineViewProps = {
  timeline: DayTimeline
  dayStartMs: number
  scale: TimelineScale
  nowMs: number | null
  selection: Selection | null
  highlight: string | null
  onSelect: (selection: Selection) => void
  /** The details card of the selection, placed next to it. */
  card: ReactNode
  scrollerRef: RefObject<HTMLDivElement | null>
  /** Shows a prompt's text in its hover card. */
  PromptText: PromptTextView
  /** Shows a commit's details in its hover card. */
  CommitDetails: CommitDetailsView
  /** Lanes folded to a strip of marks. */
  collapsed: ReadonlySet<LaneName>
  onToggleLane: (lane: LaneName) => void
}

/** Lanes from top to bottom, time left to right; scrolls both ways. */
export function HorizontalTimeline({
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
  const length = timelineLength(scale)
  const places = agentPlaces(timeline.agents)
  const parallel = Math.max(1, ...places.map((place) => place.columns))
  const lanes = horizontalLanes(parallel, collapsed, commits.length > 0)
  const dimmed = dimmedBy(timeline, highlight)
  const box = (span: { startMs: number; endMs: number }) =>
    spanBox(span, dayStartMs, scale)
  const selected = (lane: Selection["lane"], index: number) =>
    selection?.lane === lane && selection.index === index
  const selectedStart = (() => {
    if (!selection) return 0
    const item =
      selection.lane === "apps"
        ? timeline.blocks[selection.index]
        : selection.lane === "agents"
          ? timeline.agents[selection.index]
          : timeline.work[selection.index]
    return item ? box(item).start : 0
  })()
  const height = lanes.card + (card ? 340 : 24)
  const folded = (lane: Selection["lane"]) =>
    laneEntries(timeline, lane).map((entry) => ({
      key: `${entry.span.startMs}:${entry.index}`,
      box: box(entry.span),
      label: entry.label,
      selected: selected(lane, entry.index),
      dimmed: dimmed[lane](entry.index),
      onSelect: () => onSelect({ lane, index: entry.index }),
      column: lane === "agents" ? places[entry.index]?.column : undefined,
    }))
  return (
    <div className="relative min-h-0 flex-1 pb-16 max-[720px]:pb-14">
      <div
        ref={scrollerRef}
        role="region"
        aria-label="Timeline"
        className="h-full overflow-auto rounded-[18px] border border-(--glass-edge) bg-well pr-4 [scrollbar-width:thin] max-[720px]:pr-0"
      >
        <div className="relative flex" style={{ height }}>
          <div
            data-lane-labels=""
            className="sticky left-0 z-10 shrink-0 bg-well max-[720px]:w-13!"
            style={{ width: LABEL_PX }}
          >
            {(["apps", "agents", "work"] as const).map((lane) => (
              <LaneToggle
                key={lane}
                lane={lane}
                open={!collapsed.has(lane)}
                top={lanes[lane].top + 4}
                onToggle={onToggleLane}
              />
            ))}
            {commits.length > 0 && (
              <LaneLabel top={lanes.commits.top + 4}>Commits</LaneLabel>
            )}
            <LaneLabel top={lanes.prompts.top + 4}>Prompts</LaneLabel>
          </div>
          <div className="relative shrink-0" style={{ width: length }}>
            {timeMarks(scale).map((mark) => (
              <div key={mark.minute} aria-hidden="true">
                <span
                  className={`absolute top-7 w-px ${mark.hour ? "bg-t1/10" : "bg-t1/5"}`}
                  style={{ left: mark.offset, height: lanes.linesHeight }}
                />
                <span
                  className="absolute top-1 -translate-x-1/2 text-[11px] whitespace-nowrap text-t4"
                  style={{ left: mark.offset }}
                >
                  {mark.label}
                </span>
              </div>
            ))}
            <div
              aria-label="Apps"
              className="absolute inset-x-0"
              style={lanes.apps}
            >
              {collapsed.has("apps") ? (
                <CollapsedLane
                  view="horizontal"
                  tone="bg-t3"
                  items={folded("apps")}
                />
              ) : (
                timeline.blocks.map((block, index) => (
                  <AppBlock
                    key={block.startMs}
                    block={block}
                    box={box(block)}
                    view="horizontal"
                    selected={selected("apps", index)}
                    dimmed={dimmed.apps(index)}
                    onSelect={() => onSelect({ lane: "apps", index })}
                  />
                ))
              )}
            </div>
            <div
              aria-label="Agents"
              className="absolute inset-x-0"
              style={lanes.agents}
            >
              {collapsed.has("agents") ? (
                <CollapsedLane
                  view="horizontal"
                  tone="bg-t2"
                  tracks={parallel}
                  items={folded("agents")}
                />
              ) : (
                timeline.agents.map((agent, index) => (
                  <AgentBlock
                    key={`${agent.startMs}:${agent.source}:${agent.title ?? index}`}
                    block={agent}
                    box={box(agent)}
                    view="horizontal"
                    place={{
                      column: places[index]?.column ?? 0,
                      columns: parallel,
                    }}
                    selected={selected("agents", index)}
                    dimmed={dimmed.agents(index)}
                    onSelect={() => onSelect({ lane: "agents", index })}
                  />
                ))
              )}
            </div>
            {commits.length > 0 && (
              <div
                aria-label="Commits"
                className="absolute inset-x-0"
                style={lanes.commits}
              >
                <CommitMarks
                  timeline={timeline}
                  CommitDetails={CommitDetails}
                  view="horizontal"
                  dayStartMs={dayStartMs}
                  scale={scale}
                />
              </div>
            )}
            <div
              aria-label="Work context"
              className="absolute inset-x-0"
              style={lanes.work}
            >
              {collapsed.has("work") ? (
                <CollapsedLane
                  view="horizontal"
                  tone="bg-t4"
                  items={folded("work")}
                />
              ) : (
                timeline.work.map((span, index) => (
                  <WorkBlock
                    key={span.startMs}
                    span={span}
                    box={box(span)}
                    view="horizontal"
                    selected={selected("work", index)}
                    dimmed={dimmed.work(index)}
                    onSelect={() => onSelect({ lane: "work", index })}
                  />
                ))
              )}
            </div>
            <div
              aria-label="Your prompts"
              className="absolute inset-x-0"
              style={lanes.prompts}
            >
              {promptMarks(timeline).map((prompt, index) => (
                <button
                  key={prompt.atMs}
                  type="button"
                  tabIndex={-1}
                  aria-label={prompt.label}
                  onMouseEnter={() => setTip(prompt)}
                  onMouseLeave={() => setTip(null)}
                  className="absolute bottom-0 flex h-9 w-[11px] cursor-default items-end justify-center"
                  style={{
                    left: timeOffset(prompt.atMs, dayStartMs, scale) - 5,
                  }}
                >
                  <span
                    className={`w-[3px] rounded-[1px] ${tip && tip.atMs !== prompt.atMs ? "bg-t4" : "bg-t2"}`}
                    style={{ height: 14 + ((index * 7) % 22) }}
                  />
                </button>
              ))}
            </div>
            {tip && (
              <PromptTip
                prompt={tip}
                PromptText={PromptText}
                style={{
                  left: Math.max(
                    0,
                    Math.min(
                      timeOffset(tip.atMs, dayStartMs, scale) - 160,
                      length - 320,
                    ),
                  ),
                  top: lanes.prompts.top - 8,
                  transform: "translateY(-100%)",
                }}
              />
            )}
            {nowMs !== null && (
              <NowLine
                offset={timeOffset(nowMs, dayStartMs, scale)}
                nowMs={nowMs}
                height={lanes.linesHeight + 4}
              />
            )}
            {card && (
              <div
                className="absolute z-10 w-[460px] max-[720px]:fixed! max-[720px]:inset-x-2.5! max-[720px]:top-auto! max-[720px]:bottom-20! max-[720px]:max-h-[60vh]! max-[720px]:w-auto! max-[720px]:overflow-y-auto"
                style={{
                  top: lanes.card,
                  left: Math.max(
                    0,
                    Math.min(selectedStart, length - CARD_WIDTH),
                  ),
                }}
              >
                {card}
              </div>
            )}
          </div>
        </div>
      </div>
      <p className="absolute bottom-6 left-6.5 m-0 text-xs text-t5 max-[720px]:hidden">
        Scroll sideways · click a block for details
      </p>
    </div>
  )
}

const LANE_TITLES = { apps: "Apps", agents: "Agents", work: "Work" } as const

/** A lane's name that folds or unfolds the lane. */
function LaneToggle({
  lane,
  open,
  top,
  onToggle,
}: {
  lane: LaneName
  open: boolean
  top: number
  onToggle: (lane: LaneName) => void
}) {
  return (
    <button
      type="button"
      aria-expanded={open}
      aria-label={
        open ? `Fold ${LANE_TITLES[lane]}` : `Unfold ${LANE_TITLES[lane]}`
      }
      title={open ? `Fold ${LANE_TITLES[lane]}` : `Unfold ${LANE_TITLES[lane]}`}
      onClick={() => onToggle(lane)}
      className="absolute left-2 inline-flex items-center gap-1 rounded-md text-t4 outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring max-[720px]:left-0 max-[720px]:text-[11px]"
      style={{ top }}
    >
      <ChevronDownIcon
        aria-hidden="true"
        className={`size-3 transition-transform ${open ? "" : "-rotate-90"}`}
      />
      {LANE_TITLES[lane]}
    </button>
  )
}

function LaneLabel({ top, children }: { top: number; children: string }) {
  return (
    <span
      className="absolute left-6.5 text-t4 max-[720px]:left-0.5 max-[720px]:text-[11px]"
      style={{ top }}
    >
      {children}
    </span>
  )
}

function NowLine({
  offset,
  nowMs,
  height,
}: {
  offset: number
  nowMs: number
  height: number
}) {
  return (
    <>
      <span
        aria-hidden="true"
        className="absolute top-6 border-l border-dashed border-t1/60"
        style={{ left: offset, height }}
      />
      <span
        className="absolute ml-1.5 text-[11px] text-t2"
        style={{ left: offset, top: height + 16 }}
      >
        now {formatClock(nowMs)}
      </span>
    </>
  )
}
