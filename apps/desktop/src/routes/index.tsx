import { createFileRoute } from "@tanstack/react-router"
import { redirectToToday } from "@/pages/day"
import { redirectToSetupWhenNew } from "@/pages/setup"

export const Route = createFileRoute("/")({
  beforeLoad: async () => {
    await redirectToSetupWhenNew()
    redirectToToday()
  },
})
