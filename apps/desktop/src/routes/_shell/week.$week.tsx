import { createFileRoute } from "@tanstack/react-router"
import { WeekRouteComponent } from "@/pages/week"

type WeekSearch = { issue?: string }

const ISSUE_KEY = /^[A-Z][A-Z0-9]+-\d+$/

export const Route = createFileRoute("/_shell/week/$week")({
  validateSearch: (search: { issue?: string }): WeekSearch =>
    typeof search.issue === "string" && ISSUE_KEY.test(search.issue)
      ? { issue: search.issue }
      : {},
  component: WeekRouteComponent,
})
