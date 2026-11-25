import { fireEvent, render, screen } from "@testing-library/react"
import { formatDuration } from "@/shared/lib/duration"
import { describe, expect, it, vi } from "vitest"
import { PopoverPage } from "@/pages/popover/ui/popover-page"
import { sampleToday } from "@/pages/popover/ui/sample-today.fixture"

const handlers = {
  onOpen: vi.fn(),
  onPause: vi.fn(),
  onResume: vi.fn(),
  onQuit: vi.fn(),
}

describe("PopoverPage", () => {
  it("shows the window in front and today's apps", () => {
    render(
      <PopoverPage
        {...handlers}
        today={sampleToday}
        status={{
          state: {
            kind: "recording",
            current: {
              app: { id: "com.jetbrains.rider", name: "Rider" },
              title: { kind: "known", text: "CatalogSource.fs" },
              document: null,
              url: null,
            },
          },
          titleAccess: "granted",
          titlesLostMs: null,
        }}
      />,
    )

    expect(screen.getByText("CatalogSource.fs")).toBeInTheDocument()
    expect(screen.getByRole("region", { name: "Today" })).toHaveTextContent(
      `Your work today${formatDuration(sampleToday.totals.split.workMs)}`,
    )
    expect(screen.getAllByRole("listitem")).toHaveLength(7)
  })

  it("resumes instead of pausing while paused", () => {
    render(
      <PopoverPage
        {...handlers}
        today={undefined}
        status={{
          state: { kind: "paused", untilMs: 0 },
          titleAccess: "granted",
          titlesLostMs: null,
        }}
      />,
    )

    fireEvent.click(screen.getByRole("button", { name: "Resume recording" }))

    expect(handlers.onResume).toHaveBeenCalledOnce()
    expect(handlers.onPause).not.toHaveBeenCalled()
  })
})
