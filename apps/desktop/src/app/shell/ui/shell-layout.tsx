import { useRender } from "@base-ui/react/use-render"
import { SettingsIcon } from "lucide-react"
import type { ReactElement, ReactNode } from "react"
import type { RecorderStatus } from "@/shared/api/generated/RecorderStatus"
import { Logo } from "@/app/shell/ui/logo"
import { QueueBadge } from "@/app/shell/ui/queue-badge"
import { RecorderPill } from "@/app/shell/ui/recorder-pill"
import { TabLink } from "@/app/shell/ui/tab-link"
import { ThemeSwitch } from "@/app/shell/ui/theme-switch"
import { TooltipProvider } from "@/shared/ui/tooltip"

export type ShellPage = "day" | "week" | "queue" | "rules" | "settings"

type ShellLayoutProps = {
  current: ShellPage | null
  /** Waiting proposals, shown on the Log queue tab. */
  waiting: number
  status: RecorderStatus | undefined
  /** The link element for each page (router links in the app). */
  links: Record<ShellPage, ReactElement>
  /** The Needs you control. */
  needs: ReactNode
  /** The search control. */
  search: ReactNode
  /** A notice above the page, for example missing title access. */
  notice?: ReactNode
  children: ReactNode
}

/**
 * The main window: top bar (logo, tabs, recorder, Needs you, theme,
 * settings) and the page. Below 720 px the tabs move to a bottom bar.
 */
export function ShellLayout({
  current,
  waiting,
  status,
  links,
  needs,
  search,
  notice,
  children,
}: ShellLayoutProps) {
  const nav = (className: string) => (
    <MainNav
      current={current}
      waiting={waiting}
      links={links}
      className={className}
    />
  )
  return (
    <TooltipProvider>
      <div className="flex h-dvh flex-col gap-3.5 bg-(--backdrop) p-3.5 text-foreground max-[720px]:pb-20">
        <header className="flex items-center gap-2.5">
          <Logo />
          {nav("ml-2 max-[720px]:hidden")}
          <span className="ml-auto" />
          {search}
          <RecorderPill status={status} link={links.settings} />
          {needs}
          <ThemeSwitch />
          <SettingsLink active={current === "settings"} link={links.settings} />
        </header>
        <main className="flex min-h-0 min-w-0 flex-1 flex-col overflow-y-auto">
          {notice}
          {children}
        </main>
        {nav("fixed inset-x-2.5 bottom-2.5 z-30 hidden max-[720px]:flex")}
      </div>
    </TooltipProvider>
  )
}

type MainNavProps = {
  current: ShellPage | null
  waiting: number
  links: Record<ShellPage, ReactElement>
  className: string
}

function MainNav({ current, waiting, links, className }: MainNavProps) {
  return (
    <nav
      aria-label="Main"
      className={`glass flex gap-0.5 rounded-full p-[3px] ${className}`}
    >
      <TabLink label="Day" active={current === "day"} render={links.day} />
      <TabLink label="Week" active={current === "week"} render={links.week} />
      <TabLink
        label="Log queue"
        name={waiting > 0 ? `Log queue, ${waiting} waiting` : undefined}
        active={current === "queue"}
        badge={<QueueBadge count={waiting} />}
        render={links.queue}
      />
      <TabLink
        label="Apps & rules"
        active={current === "rules"}
        render={links.rules}
      />
    </nav>
  )
}

function SettingsLink({
  active,
  link,
}: {
  active: boolean
  link: ReactElement
}) {
  return useRender({
    render: link,
    props: {
      "aria-label": "Settings",
      title: "Settings (⌘,)",
      "aria-current": active ? "page" : undefined,
      className: `glass grid size-10 place-items-center rounded-full outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring ${active ? "bg-selected text-foreground" : "text-t3"}`,
      children: <SettingsIcon aria-hidden="true" className="size-4" />,
    },
  })
}
