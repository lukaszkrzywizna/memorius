import { useState } from "react"
import type { DayCommit } from "@/shared/api/generated/DayCommit"
import type { DayTimeline } from "@/shared/api/generated/DayTimeline"
import { agentApp } from "@/pages/day/lib/agent-lanes"
import type { TimelineScale } from "@/pages/day/lib/timeline-layout"
import { timeOffset } from "@/pages/day/lib/timeline-layout"
import type { TimelineView } from "@/pages/day/lib/timeline-view"
import {
  CommitCard,
  type CommitDetailsView,
} from "@/pages/day/ui/timeline/commit-card"
import { formatClock } from "@/shared/lib/local-date"

/** Room a commit's subject needs before the next node, per direction. */
const LABEL_ROOM_PX = { horizontal: 120, vertical: 16 } as const

/** What ran when a commit was made: its agent session and work span. */
function commitContext(timeline: DayTimeline, commit: DayCommit) {
  const at = (span: { startMs: number; endMs: number }) =>
    span.startMs <= commit.atMs && commit.atMs < span.endMs
  const sessions = timeline.agents.filter(at)
  const session =
    sessions.find((agent) => agent.branch === commit.branch) ?? sessions[0]
  const work = timeline.work.find(at)
  const parts = [
    session &&
      `During ${agentApp(session.source).name}: ${session.title ?? session.branch ?? "a session"}`,
    work &&
      `Work: ${work.confidence === "medium" ? `${work.issue}?` : work.issue}`,
  ].filter(Boolean)
  return parts.length > 0 ? parts.join(" · ") : null
}

/**
 * The Commits lane: a thin track with a node at each commit's time and its
 * subject beside the node when there is room; hovering a node opens its
 * card with where, when and what changed.
 */
export function CommitMarks({
  timeline,
  view,
  dayStartMs,
  scale,
  CommitDetails,
}: {
  timeline: DayTimeline
  view: TimelineView
  dayStartMs: number
  scale: TimelineScale
  CommitDetails: CommitDetailsView
}) {
  const { commits } = timeline
  const [hover, setHover] = useState<number | null>(null)
  const horizontal = view === "horizontal"
  const offsets = commits.map((commit) =>
    timeOffset(commit.atMs, dayStartMs, scale),
  )
  return (
    <>
      <span
        aria-hidden="true"
        className={`absolute bg-t1/12 ${
          horizontal ? "inset-x-0 top-1/2 h-px" : "inset-y-0 left-[9px] w-px"
        }`}
      />
      {commits.map((commit, index) => {
        const offset = offsets[index] ?? 0
        const next = offsets[index + 1]
        const room = next === undefined || next - offset >= LABEL_ROOM_PX[view]
        const open = hover === index
        return (
          <div
            key={commit.hash}
            className={`absolute ${open ? "z-30" : ""} ${
              horizontal ? "top-1/2 -translate-y-1/2" : "left-[4px]"
            }`}
            style={horizontal ? { left: offset - 5 } : { top: offset - 5 }}
            onMouseEnter={() => setHover(index)}
            onMouseLeave={() => setHover(null)}
          >
            <button
              type="button"
              aria-label={`${formatClock(commit.atMs)} · ${commit.subject} · ${commit.repo}`}
              aria-expanded={open}
              onFocus={() => setHover(index)}
              onClick={() => setHover(open ? null : index)}
              className="flex items-center gap-1.5 rounded-full outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <span
                aria-hidden="true"
                className={`size-2.5 shrink-0 rounded-full border-2 bg-well ${
                  open ? "border-foreground" : "border-t2"
                }`}
              />
              {room && (
                <span
                  className={`truncate text-[11px] text-t3 ${
                    horizontal ? "max-w-[110px]" : "max-w-[130px]"
                  }`}
                >
                  {commit.subject}
                </span>
              )}
            </button>
            {open && (
              <CommitCard
                commit={commit}
                context={commitContext(timeline, commit)}
                CommitDetails={CommitDetails}
                className={
                  horizontal ? "top-full left-0 mt-1.5" : "top-0 left-6"
                }
              />
            )}
          </div>
        )
      })}
    </>
  )
}
