import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useNavigate, useParams, useSearch } from "@tanstack/react-router"
import { DEFAULT_VIEW } from "@/pages/day/lib/timeline-view"
import { DEFAULT_ZOOM } from "@/pages/day/lib/zoom"
import { dayTimelineState } from "@/pages/day/model/day-timeline-state"
import { useNow } from "@/pages/day/model/use-now"
import { readStoredView, storeView } from "@/pages/day/model/view-storage"
import { readStoredZoom, storeZoom } from "@/pages/day/model/zoom-storage"
import { DayPage } from "@/pages/day/ui/day-page"
import { CommitDetailsFromGit } from "@/pages/day/ui/commit-details-from-git"
import { PromptTextFromLog } from "@/pages/day/ui/prompt-text-from-log"
import {
  dayTimelineQuery,
  memoriusQueryKey,
  openLink,
  setSessionsPrivate,
  trackerStatusQuery,
  weekSummaryQuery,
} from "@/shared/api/memorius-commands"
import { todayIso } from "@/shared/lib/local-date"

export function DayRouteComponent() {
  const { date } = useParams({ from: "/_shell/day/$date" })
  const search = useSearch({ from: "/_shell/day/$date" })
  const zoom = search.zoom ?? readStoredZoom() ?? DEFAULT_ZOOM
  const view = search.view ?? readStoredView() ?? DEFAULT_VIEW
  const highlight = search.issue ?? null
  const navigate = useNavigate()
  const nowMs = useNow()
  const query = useQuery(dayTimelineQuery(date, zoom))
  const week = useQuery(weekSummaryQuery(date)).data
  const tracker = useQuery(trackerStatusQuery()).data?.url
  const queryClient = useQueryClient()
  // A private session changes the day, the week and what agents may propose.
  const markPrivate = useMutation({
    mutationFn: setSessionsPrivate,
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: memoriusQueryKey }),
  })
  const go = (next: {
    date?: string
    zoom?: typeof zoom
    view?: typeof view
    issue?: string | null
  }) =>
    void navigate({
      to: "/day/$date",
      params: { date: next.date ?? date },
      search: {
        zoom: next.zoom ?? zoom,
        view: next.view ?? view,
        issue: (next.issue === undefined ? highlight : next.issue) ?? undefined,
      },
    })

  return (
    <DayPage
      date={date}
      today={todayIso()}
      state={dayTimelineState(query)}
      onDateChange={(next) => go({ date: next })}
      zoom={zoom}
      onZoomChange={(next) => {
        storeZoom(next)
        go({ zoom: next })
      }}
      view={view}
      onViewChange={(next) => {
        storeView(next)
        go({ view: next })
      }}
      unloggedMs={week?.days.find((day) => day.date === date)?.unloggedMs}
      nowMs={nowMs}
      highlight={highlight}
      onClearHighlight={() => go({ issue: null })}
      onMarkPrivate={(request) => markPrivate.mutate(request)}
      onOpenIssue={
        tracker
          ? (issue) => void openLink(`${tracker}/issue/${issue}`)
          : undefined
      }
      PromptText={PromptTextFromLog}
      CommitDetails={CommitDetailsFromGit}
    />
  )
}
