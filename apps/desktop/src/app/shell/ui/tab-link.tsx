import { useRender } from "@base-ui/react/use-render"
import type { ReactElement, ReactNode } from "react"

type TabLinkProps = {
  label: string
  /** The accessible name when it says more than the label, for example a count. */
  name?: string
  active: boolean
  badge?: ReactNode
  /** The link element that the tab renders as. */
  render: ReactElement
}

/** A page tab in the top bar or the bottom tab bar. */
export function TabLink({ label, name, active, badge, render }: TabLinkProps) {
  return useRender({
    render,
    props: {
      "aria-label": name,
      "aria-current": active ? "page" : undefined,
      className: `inline-flex h-9 items-center justify-center gap-1.5 rounded-full px-4 whitespace-nowrap outline-none focus-visible:ring-2 focus-visible:ring-ring max-[720px]:flex-1 max-[720px]:px-1 max-[720px]:text-xs ${
        active ? "bg-accent text-foreground" : "text-t4 hover:text-foreground"
      }`,
      children: (
        <>
          {label}
          {badge}
        </>
      ),
    },
  })
}
