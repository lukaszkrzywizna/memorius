import type { WeekState } from "@/pages/week/model/week-state"
import {
  visibleDays,
  weekRange,
  weekTitle,
  weeksAgo,
} from "@/pages/week/lib/week-days"
import { DayStrip } from "@/pages/week/ui/day-strip"
import { Timesheet } from "@/pages/week/ui/timesheet"
import { WeekHeader } from "@/pages/week/ui/week-header"
import { WeekSkeleton } from "@/pages/week/ui/week-skeleton"
import { StatCards, StatCardsSkeleton } from "@/shared/ui/stat-cards"

type WeekPageProps = {
  week: string
  today: string
  state: WeekState
  onWeekChange: (date: string) => void
  onOpenDay: (date: string) => void
  /** An issue to mark in the timesheet. */
  highlight: string | null
}

export function WeekPage({
  week,
  today,
  state,
  onWeekChange,
  onOpenDay,
  highlight,
}: WeekPageProps) {
  const days = state.kind === "ready" ? state.summary.days : []
  const sum = (pick: (day: (typeof days)[number]) => number) =>
    days.reduce((total, day) => total + pick(day), 0)
  const relative = weeksAgo(days, today)
  return (
    <div className="flex flex-col gap-4 px-1 pt-1.5 pb-6">
      <WeekHeader
        week={week}
        title={weekTitle(days)}
        subtitle={[weekRange(days), relative].filter(Boolean).join(" · ")}
        thisWeek={relative === "this week" || days.length === 0}
        today={today}
        onWeekChange={onWeekChange}
        totals={
          state.kind === "ready" ? (
            <StatCards
              label="Week totals"
              stats={[
                {
                  label: "Tracked",
                  ms: sum((day) => day.activeMs),
                  dot: "bg-operational",
                },
                { label: "Logged", ms: sum((day) => day.loggedMs) },
                {
                  label: "Proposed",
                  ms: sum((day) => day.proposedMs),
                  dot: "bg-attention",
                },
                { label: "Not logged", ms: sum((day) => day.unloggedMs) },
              ]}
            />
          ) : state.kind === "loading" ? (
            <StatCardsSkeleton count={4} />
          ) : null
        }
      />
      <WeekContent
        state={state}
        today={today}
        onOpenDay={onOpenDay}
        highlight={highlight}
      />
    </div>
  )
}

function WeekContent({
  state,
  today,
  onOpenDay,
  highlight,
}: Pick<WeekPageProps, "state" | "today" | "onOpenDay" | "highlight">) {
  switch (state.kind) {
    case "loading":
      return <WeekSkeleton />
    case "error":
      return (
        <p role="alert" className="m-0 text-destructive">
          Could not load the week: {state.message}
        </p>
      )
    case "ready":
      return (
        <>
          <DayStrip
            days={visibleDays(state.summary.days).map(({ day }) => day)}
            today={today}
            onOpenDay={onOpenDay}
          />
          {state.summary.issues.length === 0 ? (
            <p className="glass m-0 rounded-[20px] px-6 py-5 text-t4">
              No issue was detected or logged this week.
            </p>
          ) : (
            <Timesheet summary={state.summary} highlight={highlight} />
          )}
        </>
      )
  }
}
