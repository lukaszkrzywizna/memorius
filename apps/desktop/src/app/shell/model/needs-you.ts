import { useQuery } from "@tanstack/react-query"
import type { Proposal } from "@/shared/api/generated/Proposal"
import type { WeekSummary } from "@/shared/api/generated/WeekSummary"
import { logQueueQuery, weekSummaryQuery } from "@/shared/api/memorius-commands"
import { todayIso } from "@/shared/lib/local-date"

type UnloggedIssue = { issue: string; minutes: number }

/** What waits for the user today: proposals and unlogged issues. */
export type NeedsYou = {
  waiting: Proposal[]
  unlogged: UnloggedIssue[]
  unloggedMs: number
  count: number
}

export function useNeedsYou(): NeedsYou {
  const today = todayIso()
  const queue = useQuery(logQueueQuery()).data
  const week = useQuery(weekSummaryQuery(today)).data
  return needsYou(today, {
    proposals: queue?.map((item) => item.proposal) ?? [],
    week,
  })
}

export function needsYou(
  today: string,
  data: {
    proposals: Proposal[]
    week: WeekSummary | undefined
  },
): NeedsYou {
  const waiting = data.proposals.filter(
    (proposal) => proposal.state.kind === "proposed",
  )
  const index = data.week?.days.findIndex((day) => day.date === today) ?? -1
  const unlogged =
    index < 0
      ? []
      : (data.week?.issues ?? [])
          .flatMap(({ issue, cells }) => {
            const cell = cells[index]
            return cell?.kind === "detected"
              ? [{ issue, minutes: cell.minutes }]
              : []
          })
          .sort((a, b) => b.minutes - a.minutes)
  return {
    waiting,
    unlogged,
    unloggedMs: index < 0 ? 0 : (data.week?.days[index]?.unloggedMs ?? 0),
    count: waiting.length,
  }
}
