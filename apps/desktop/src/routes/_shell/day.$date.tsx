import { createFileRoute } from "@tanstack/react-router"
import {
  DayRouteComponent,
  type TimelineView,
  type ZoomMinutes,
  parseTimelineView,
  parseZoom,
} from "@/pages/day"

type DaySearch = { zoom?: ZoomMinutes; view?: TimelineView; issue?: string }

const ISSUE_KEY = /^[A-Z][A-Z0-9]+-\d+$/

export const Route = createFileRoute("/_shell/day/$date")({
  validateSearch: (search: {
    zoom?: string | number
    view?: string
    issue?: string
  }): DaySearch => {
    const zoom = parseZoom(search.zoom)
    const view = parseTimelineView(search.view)
    const issue =
      typeof search.issue === "string" && ISSUE_KEY.test(search.issue)
        ? search.issue
        : undefined
    return {
      ...(zoom ? { zoom } : {}),
      ...(view ? { view } : {}),
      ...(issue ? { issue } : {}),
    }
  },
  component: DayRouteComponent,
})
