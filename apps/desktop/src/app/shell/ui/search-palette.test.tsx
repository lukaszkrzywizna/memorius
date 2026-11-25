import { fireEvent, render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { resultGroups } from "@/app/shell/model/search-items"
import { sampleSearch } from "@/app/shell/ui/search.fixture"
import { SearchPalette } from "@/app/shell/ui/search-palette"

function mount(state: "idle" | "ready" = "ready") {
  const onPick = vi.fn()
  render(
    <SearchPalette
      query="is-61"
      onQueryChange={vi.fn()}
      groups={resultGroups(sampleSearch).groups}
      dates={[]}
      state={state}
      truncated={false}
      onPick={onPick}
    />,
  )
  return onPick
}

describe("SearchPalette", () => {
  it("moves with the arrows and opens with Enter", () => {
    const onPick = mount()
    const input = screen.getByRole("combobox")

    fireEvent.keyDown(input, { key: "ArrowDown" })
    expect(screen.getAllByRole("option")[1]).toHaveAttribute(
      "aria-selected",
      "true",
    )
    fireEvent.keyDown(input, { key: "Enter" })

    expect(onPick).toHaveBeenCalledWith({
      page: "day",
      date: "2026-10-09",
      issue: "IS-61",
    })
  })

  it("opens a day from an issue's chip", () => {
    const onPick = mount()

    fireEvent.click(screen.getByRole("button", { name: "Wed 7 Oct · 40m" }))

    expect(onPick).toHaveBeenCalledWith({
      page: "day",
      date: "2026-10-07",
      issue: "IS-61",
    })
  })

  it("says what to type before a search", () => {
    mount("idle")

    expect(screen.getByRole("status")).toHaveTextContent(/Type an issue key/)
  })
})
