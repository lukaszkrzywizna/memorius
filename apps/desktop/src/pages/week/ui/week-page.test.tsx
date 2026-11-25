import { fireEvent, render, screen, within } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { WeekPage } from "@/pages/week/ui/week-page"
import { sampleWeek } from "@/pages/week/ui/sample-week.fixture"

const ready = { kind: "ready" as const, summary: sampleWeek }

describe("WeekPage", () => {
  it("shows weekdays and one row per issue with its total", () => {
    render(
      <WeekPage
        week="2026-10-09"
        today="2026-10-11"
        state={ready}
        onWeekChange={vi.fn()}
        onOpenDay={vi.fn()}
        highlight={null}
      />,
    )

    expect(screen.getByRole("heading", { level: 1 })).toHaveTextContent(
      "Week 41",
    )
    const table = screen.getByRole("table")
    expect(within(table).getAllByRole("columnheader")).toHaveLength(7)
    const row = within(table).getByRole("row", { name: /INT-290/ })
    expect(within(row).getByText("7h 52m")).toBeInTheDocument()
  })

  it("opens a day and moves between weeks", () => {
    const onWeekChange = vi.fn()
    const onOpenDay = vi.fn()
    render(
      <WeekPage
        week="2026-10-09"
        today="2026-10-11"
        state={ready}
        onWeekChange={onWeekChange}
        onOpenDay={onOpenDay}
        highlight={null}
      />,
    )

    fireEvent.click(
      within(screen.getByLabelText("Days")).getAllByRole("button")[0]!,
    )
    fireEvent.click(screen.getByRole("button", { name: "Previous week" }))

    expect(onOpenDay).toHaveBeenCalledWith("2026-10-05")
    expect(onWeekChange).toHaveBeenCalledWith("2026-10-02")
    expect(screen.getByRole("button", { name: "Next week" })).toBeDisabled()
  })
})
