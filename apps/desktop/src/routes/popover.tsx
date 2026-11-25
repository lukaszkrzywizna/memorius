import { createFileRoute } from "@tanstack/react-router"
import { PopoverRouteComponent } from "@/pages/popover"

export const Route = createFileRoute("/popover")({
  component: PopoverRouteComponent,
})
