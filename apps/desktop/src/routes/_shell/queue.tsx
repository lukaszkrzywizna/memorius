import { createFileRoute } from "@tanstack/react-router"
import { LogQueueRouteComponent } from "@/pages/log-queue"

export const Route = createFileRoute("/_shell/queue")({
  component: LogQueueRouteComponent,
})
