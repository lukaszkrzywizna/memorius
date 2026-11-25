import { useHotkey } from "@tanstack/react-hotkeys"
import { useEffect, useEffectEvent, useRef, useState } from "react"
import type { DayTimeline } from "@/shared/api/generated/DayTimeline"
import type { SessionsPrivateRequest } from "@/shared/api/memorius-commands"
import {
  type TimelineScale,
  timeOffset,
  timelineScale,
} from "@/pages/day/lib/timeline-layout"
import type { LaneName } from "@/pages/day/lib/lane-widths"
import type { TimelineView } from "@/pages/day/lib/timeline-view"
import type { ZoomMinutes } from "@/pages/day/lib/zoom"
import {
  readStoredCollapsedLanes,
  storeCollapsedLanes,
} from "@/pages/day/model/lane-widths-storage"
import { SelectionDetails } from "@/pages/day/ui/details/selection-details"
import type { CommitDetailsView } from "@/pages/day/ui/timeline/commit-card"
import { HorizontalTimeline } from "@/pages/day/ui/timeline/horizontal-timeline"
import type { PromptTextView } from "@/pages/day/ui/timeline/prompt-tip"
import {
  type Selection,
  selectionStartMs,
} from "@/pages/day/ui/timeline/timeline-model"
import { VerticalTimeline } from "@/pages/day/ui/timeline/vertical-timeline"
import { ZoomBar } from "@/pages/day/ui/zoom-bar"

type DayReadyProps = {
  timeline: DayTimeline
  dayStartMs: number
  zoom: ZoomMinutes
  onZoomChange: (zoom: ZoomMinutes) => void
  view: TimelineView
  onViewChange: (view: TimelineView) => void
  /** The current time when the day is today. */
  nowMs: number | null
  highlight: string | null
  onMarkPrivate: (decision: SessionsPrivateRequest) => void
  PromptText: PromptTextView
  CommitDetails: CommitDetailsView
}

export function DayReady({
  timeline,
  dayStartMs,
  zoom,
  onZoomChange,
  view,
  onViewChange,
  nowMs,
  highlight,
  onMarkPrivate,
  PromptText,
  CommitDetails,
}: DayReadyProps) {
  const [selection, setSelection] = useState<Selection | null>(null)
  const [collapsed, setCollapsed] = useState<ReadonlySet<LaneName>>(
    () => new Set(readStoredCollapsedLanes()),
  )
  const toggleLane = (lane: LaneName) =>
    setCollapsed((current) => {
      const next = new Set(current)
      if (next.has(lane)) next.delete(lane)
      else next.add(lane)
      storeCollapsedLanes(next)
      return next
    })
  const scrollerRef = useRef<HTMLDivElement | null>(null)
  const scale = timelineScale(
    [...timeline.blocks, ...timeline.agents],
    dayStartMs,
    zoom,
    view,
  )
  const firstHighlighted = highlight
    ? timeline.work.find((span) => span.issue === highlight)?.startMs
    : undefined
  const focusMs =
    selectionStartMs(timeline, selection) ??
    firstHighlighted ??
    nowMs ??
    timeline.blocks[0]?.startMs

  // Only a new view, zoom or highlight moves the timeline by itself; a new
  // selection must not, so the target is read in an effect event.
  const revealFocus = useEffectEvent(() =>
    reveal(scrollerRef.current, focusMs, dayStartMs, scale, false),
  )
  useEffect(
    function revealFocusOnViewChange() {
      revealFocus()
    },
    [view, zoom, highlight],
  )

  const open = selection !== null
  useHotkey("Escape", () => setSelection(null), { enabled: open })
  useEffect(
    function closeDetailsOnOutsideClick() {
      if (!open) return
      const close = (event: PointerEvent) => {
        // The card keeps its own clicks; a block click selects that block.
        if (
          event.target instanceof Element &&
          event.target.closest('[role="dialog"], [aria-pressed]')
        )
          return
        setSelection(null)
      }
      document.addEventListener("pointerdown", close)
      return () => document.removeEventListener("pointerdown", close)
    },
    [open],
  )

  if (timeline.blocks.length === 0 && timeline.agents.length === 0) {
    return (
      <p className="glass mx-6 rounded-[20px] px-6 py-5 text-t4">
        No activity was recorded on this day.
      </p>
    )
  }
  const Timeline = view === "horizontal" ? HorizontalTimeline : VerticalTimeline
  return (
    <div className="relative flex min-h-0 flex-1 flex-col">
      <Timeline
        timeline={timeline}
        dayStartMs={dayStartMs}
        scale={scale}
        nowMs={nowMs}
        selection={selection}
        highlight={highlight}
        onSelect={setSelection}
        scrollerRef={scrollerRef}
        PromptText={PromptText}
        CommitDetails={CommitDetails}
        collapsed={collapsed}
        onToggleLane={toggleLane}
        card={
          selection && (
            <SelectionDetails
              timeline={timeline}
              selection={selection}
              onMarkPrivate={onMarkPrivate}
              onClose={() => setSelection(null)}
            />
          )
        }
      />
      <ZoomBar
        zoom={zoom}
        onZoomChange={onZoomChange}
        view={view}
        onViewChange={onViewChange}
        onNow={
          nowMs === null
            ? undefined
            : () => reveal(scrollerRef.current, nowMs, dayStartMs, scale, true)
        }
      />
    </div>
  )
}

/** Scrolls the timeline so the moment sits a third into the view. */
function reveal(
  scroller: HTMLDivElement | null,
  ms: number | undefined,
  dayStartMs: number,
  scale: TimelineScale,
  smooth: boolean,
) {
  if (!scroller || ms === undefined) return
  const offset = timeOffset(ms, dayStartMs, scale)
  const behavior = smooth ? "smooth" : "auto"
  if (scale.view === "horizontal") {
    // The lane names stay pinned at the left of the scrolled content.
    const labels =
      scroller.querySelector<HTMLElement>("[data-lane-labels]")?.offsetWidth ??
      0
    scroller.scrollTo?.({
      left: Math.max(0, labels + offset - scroller.clientWidth * 0.3),
      behavior,
    })
  } else {
    scroller.scrollTo?.({
      top: Math.max(0, offset - scroller.clientHeight * 0.3),
      behavior,
    })
  }
}
