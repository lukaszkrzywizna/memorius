import { createFileRoute } from "@tanstack/react-router"
import { SetupRouteComponent } from "@/pages/setup"

export const Route = createFileRoute("/setup")({
  component: SetupRouteComponent,
})
