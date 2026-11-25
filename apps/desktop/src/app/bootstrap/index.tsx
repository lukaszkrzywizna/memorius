import { StrictMode } from "react"
import { createRoot } from "react-dom/client"
import { AppProviders } from "@/app/providers/app-providers"
import { queryClient } from "@/app/providers/query-client"
import { memoriusQueryKey } from "@/shared/api/memorius-commands"
import { onActivityChanged } from "@/shared/api/memorius-events"
import { startTheme } from "@/shared/lib/theme"

export function bootstrap() {
  const rootElement = document.getElementById("root")
  if (!rootElement) {
    throw new Error("Missing #root element in index.html")
  }
  startTheme()
  // A new activity row or a recording state change refreshes every view.
  void onActivityChanged(() => {
    void queryClient.invalidateQueries({ queryKey: memoriusQueryKey })
  })
  createRoot(rootElement).render(
    <StrictMode>
      <AppProviders />
    </StrictMode>,
  )
}
