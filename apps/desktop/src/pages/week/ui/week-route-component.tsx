import { useQuery } from "@tanstack/react-query"
import { useNavigate, useParams, useSearch } from "@tanstack/react-router"
import { weekState } from "@/pages/week/model/week-state"
import { WeekPage } from "@/pages/week/ui/week-page"
import { weekSummaryQuery } from "@/shared/api/memorius-commands"
import { todayIso } from "@/shared/lib/local-date"

export function WeekRouteComponent() {
  const { week } = useParams({ from: "/_shell/week/$week" })
  const { issue } = useSearch({ from: "/_shell/week/$week" })
  const navigate = useNavigate()
  const query = useQuery(weekSummaryQuery(week))

  return (
    <WeekPage
      week={week}
      today={todayIso()}
      state={weekState(query)}
      highlight={issue ?? null}
      onWeekChange={(next) =>
        void navigate({ to: "/week/$week", params: { week: next } })
      }
      onOpenDay={(date) =>
        void navigate({
          to: "/day/$date",
          params: { date },
          search: issue ? { issue } : {},
        })
      }
    />
  )
}
