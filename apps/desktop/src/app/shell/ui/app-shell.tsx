import { useHotkey } from "@tanstack/react-hotkeys"
import { useQuery } from "@tanstack/react-query"
import {
  Link,
  Outlet,
  useMatchRoute,
  useNavigate,
} from "@tanstack/react-router"
import { useWaitingProposals } from "@/app/shell/model/waiting-proposals"
import { NeedsYouButton } from "@/app/shell/ui/needs-you-button"
import { SearchButton } from "@/app/shell/ui/search-button"
import { type ShellPage, ShellLayout } from "@/app/shell/ui/shell-layout"
import { SetupNotice } from "@/app/shell/ui/setup-notice"
import { recorderStatusQuery } from "@/shared/api/memorius-commands"
import { todayIso } from "@/shared/lib/local-date"

export function AppShell() {
  const status = useQuery(recorderStatusQuery()).data
  const waiting = useWaitingProposals()
  const navigate = useNavigate()
  // The macOS convention for an app's settings.
  useHotkey("Mod+,", () => void navigate({ to: "/settings" }))
  return (
    <ShellLayout
      current={useCurrentPage()}
      waiting={waiting}
      status={status}
      links={{
        day: <Link to="/day/$date" params={{ date: todayIso() }} />,
        week: <Link to="/week/$week" params={{ week: todayIso() }} />,
        queue: <Link to="/queue" />,
        rules: <Link to="/rules" />,
        settings: <Link to="/settings" />,
      }}
      needs={<NeedsYouButton />}
      search={<SearchButton />}
      notice={<SetupNotice />}
    >
      <Outlet />
    </ShellLayout>
  )
}

function useCurrentPage(): ShellPage | null {
  const matchRoute = useMatchRoute()
  if (matchRoute({ to: "/day/$date", fuzzy: true })) return "day"
  if (matchRoute({ to: "/week/$week", fuzzy: true })) return "week"
  if (matchRoute({ to: "/queue" })) return "queue"
  if (matchRoute({ to: "/rules" })) return "rules"
  if (matchRoute({ to: "/settings" })) return "settings"
  return null
}
