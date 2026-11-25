import { fireEvent, render, screen } from "@testing-library/react"
import { createElement } from "react"
import { describe, expect, it } from "vitest"
import type { RecorderStatus } from "@/shared/api/generated/RecorderStatus"
import { RecorderPill } from "@/app/shell/ui/recorder-pill"
import { TooltipProvider } from "@/shared/ui/tooltip"

const rider = {
  app: { id: "com.jetbrains.rider", name: "Rider" },
  title: { kind: "known", text: "OrderSync.fs" },
  document: null,
  url: null,
} as const

/** The dot gets its name from the trigger, so the link has no content here. */
const settingsLink = createElement("a", { href: "#/settings" })

function mountDot(status: RecorderStatus | undefined) {
  render(
    <TooltipProvider>
      <RecorderPill status={status} link={settingsLink} />
    </TooltipProvider>,
  )
  return screen.getByRole("link")
}

describe("RecorderPill", () => {
  it.each([
    [
      "recording",
      {
        state: { kind: "recording", current: rider },
        titleAccess: "granted",
        titlesLostMs: null,
      },
      "Recording",
      "Recording · Rider · OrderSync.fs",
    ],
    [
      "titles hidden",
      {
        state: { kind: "recording", current: rider },
        titleAccess: "denied",
        titlesLostMs: null,
      },
      "Recording, titles hidden",
      "Recording, titles hidden: allow Accessibility",
    ],
    [
      "paused",
      {
        state: { kind: "paused", untilMs: new Date(2026, 9, 15, 14).getTime() },
        titleAccess: "granted",
        titlesLostMs: null,
      },
      "Paused until 14:00",
      "Paused until 14:00",
    ],
  ] as const)(
    "is a focusable settings link with the %s state and its details on focus",
    async (_, status, state, detail) => {
      const link = mountDot(status)

      expect(link).toHaveAccessibleName(`${state}. Recorder settings`)
      expect(link).toHaveAttribute("href", "#/settings")
      expect(screen.getByRole("status")).toHaveTextContent(state)
      link.focus()
      fireEvent.focus(link)
      // The pill can show the same words, so look inside the tooltip.
      expect(
        await screen.findByText(detail, {
          selector: '[data-slot="tooltip-content"]',
        }),
      ).toBeInTheDocument()
    },
  )

  it("speaks state changes only, not window changes", () => {
    const dot = (status: RecorderStatus) => (
      <TooltipProvider>
        <RecorderPill status={status} link={settingsLink} />
      </TooltipProvider>
    )
    const { rerender } = render(
      dot({
        state: { kind: "recording", current: rider },
        titleAccess: "granted",
        titlesLostMs: null,
      }),
    )
    const live = screen.getByRole("status")

    rerender(
      dot({
        state: {
          kind: "recording",
          current: { ...rider, title: { kind: "known", text: "Other.fs" } },
        },
        titleAccess: "granted",
        titlesLostMs: null,
      }),
    )
    expect(live).toHaveTextContent(/^Recording$/)

    rerender(
      dot({
        state: { kind: "paused", untilMs: new Date(2026, 9, 15, 14).getTime() },
        titleAccess: "granted",
        titlesLostMs: null,
      }),
    )
    expect(live).toHaveTextContent("Paused until 14:00")
  })

  it("does not put the current window into the live region", () => {
    mountDot({
      state: { kind: "recording", current: rider },
      titleAccess: "granted",
      titlesLostMs: null,
    })

    expect(screen.getByRole("status")).toHaveTextContent(/^Recording$/)
  })

  it("shows the app in front next to the state", () => {
    mountDot({
      state: { kind: "recording", current: rider },
      titleAccess: "granted",
      titlesLostMs: null,
    })

    expect(screen.getByRole("link")).toHaveTextContent("Recording · Rider")
  })
})
