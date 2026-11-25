import { createFileRoute } from "@tanstack/react-router"
import { SettingsRouteComponent } from "@/pages/settings"

export const Route = createFileRoute("/_shell/settings")({
  component: SettingsRouteComponent,
})
