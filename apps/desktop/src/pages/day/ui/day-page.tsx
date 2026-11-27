import { XIcon } from "lucide-react"
import type { TimelineView } from "@/pages/day/lib/timeline-view"
import type { ZoomMinutes } from "@/pages/day/lib/zoom"
import type { DayTimelineState } from "@/pages/day/model/day-timeline-state"
import type { SessionsPrivateRequest } from "@/shared/api/memorius-commands"
import { DayHeader } from "@/pages/day/ui/day-header"
import { DayReady } from "@/pages/day/ui/day-ready"
import { DaySkeleton } from "@/pages/day/ui/day-skeleton"
import { DayTotals } from "@/pages/day/ui/day-totals"
import type { CommitDetailsView } from "@/pages/day/ui/timeline/commit-card"
import type { PromptTextView } from "@/pages/day/ui/timeline/prompt-tip"
import { parseIsoDate } from "@/shared/lib/local-date"
import { StatCardsSkeleton } from "@/shared/ui/stat-cards"

type DayPageProps = {
  date: string
  today: string
  state: DayTimelineState
  onDateChange: (date: string) => void
  zoom: ZoomMinutes
  onZoomChange: (zoom: ZoomMinutes) => void
  view: TimelineView
  onViewChange: (view: TimelineView) => void
  /** The day's active time with no proposal, from the week data. */
  unloggedMs: number | undefined
  /** Now, for the "now" line; used only when the day is today. */
  nowMs: number
  /** An issue to show; the rest of the day steps back. */
  highlight: string | null
  onClearHighlight: () => void
  onMarkPrivate: (decision: SessionsPrivateRequest) => void
  /** Opens an issue in YouTrack; missing when no tracker is set up. */
  onOpenIssue?: (issue: string) => void
  /** Shows a prompt's text in its hover card (read from the session log). */
  PromptText: PromptTextView
  /** Shows a commit's details in its hover card (read from git). */
  CommitDetails: CommitDetailsView
}

export function DayPage({
  date,
  today,
  state,
  onDateChange,
  zoom,
  onZoomChange,
  view,
  onViewChange,
  unloggedMs,
  nowMs,
  highlight,
  onClearHighlight,
  onMarkPrivate,
  onOpenIssue,
  PromptText,
  CommitDetails,
}: DayPageProps) {
  return (
    <div className="flex h-full min-h-0 flex-col">
      <DayHeader
        date={date}
        today={today}
        onDateChange={onDateChange}
        totals={
          state.kind === "ready" ? (
            <DayTotals totals={state.timeline.totals} unloggedMs={unloggedMs} />
          ) : state.kind === "loading" ? (
            <StatCardsSkeleton count={4} />
          ) : null
        }
        extra={
          highlight && (
            <button
              type="button"
              aria-label={`Stop showing ${highlight}`}
              onClick={onClearHighlight}
              className="inline-flex h-6 items-center gap-1.5 rounded-full border border-ring/40 pr-1.5 pl-2.5 text-xs text-t2 outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              Showing {highlight}
              <XIcon aria-hidden="true" className="size-2.5" />
            </button>
          )
        }
      />
      <DayContent
        key={date}
        date={date}
        state={state}
        zoom={zoom}
        onZoomChange={onZoomChange}
        view={view}
        onViewChange={onViewChange}
        nowMs={date === today ? nowMs : null}
        highlight={highlight}
        onMarkPrivate={onMarkPrivate}
        onOpenIssue={onOpenIssue}
        PromptText={PromptText}
        CommitDetails={CommitDetails}
      />
    </div>
  )
}

type DayContentProps = Pick<
  DayPageProps,
  | "date"
  | "state"
  | "zoom"
  | "onZoomChange"
  | "view"
  | "onViewChange"
  | "highlight"
  | "onMarkPrivate"
  | "onOpenIssue"
  | "PromptText"
  | "CommitDetails"
> & { nowMs: number | null }

function DayContent({ date, state, ...rest }: DayContentProps) {
  switch (state.kind) {
    case "loading":
      return <DaySkeleton />
    case "error":
      return (
        <p role="alert" className="m-0 px-6 text-destructive">
          Could not load activity: {state.message}
        </p>
      )
    case "ready":
      return (
        <DayReady
          timeline={state.timeline}
          dayStartMs={parseIsoDate(date).getTime()}
          {...rest}
        />
      )
  }
}
