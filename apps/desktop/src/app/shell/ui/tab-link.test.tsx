import { render, screen } from "@testing-library/react"
import { createElement } from "react"
import { describe, expect, it } from "vitest"
import { QueueBadge } from "@/app/shell/ui/queue-badge"
import { TabLink } from "@/app/shell/ui/tab-link"

const queueLink = createElement("a", { href: "#/queue" })

describe("TabLink", () => {
  it("names the tab with its count and marks the current page", () => {
    render(
      <TabLink
        label="Log queue"
        name="Log queue, 3 waiting"
        active
        badge={<QueueBadge count={3} />}
        render={queueLink}
      />,
    )

    const link = screen.getByRole("link", { name: "Log queue, 3 waiting" })
    expect(link).toHaveAttribute("href", "#/queue")
    expect(link).toHaveAttribute("aria-current", "page")
  })

  it("shows the label and no badge without a count", () => {
    render(
      <TabLink
        label="Log queue"
        active={false}
        badge={<QueueBadge count={0} />}
        render={queueLink}
      />,
    )

    const link = screen.getByRole("link", { name: "Log queue" })
    expect(link).not.toHaveAttribute("aria-current")
    expect(link).toHaveTextContent(/^Log queue$/)
  })
})
