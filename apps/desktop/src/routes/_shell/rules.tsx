import { createFileRoute } from "@tanstack/react-router"
import { RulesRouteComponent } from "@/pages/rules"

export const Route = createFileRoute("/_shell/rules")({
  component: RulesRouteComponent,
})
