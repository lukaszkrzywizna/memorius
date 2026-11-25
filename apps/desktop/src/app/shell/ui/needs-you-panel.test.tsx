import { fireEvent, render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { NeedsYouPanel } from "@/app/shell/ui/needs-you-panel"
import { nothingNeeded, sampleNeeds } from "@/app/shell/ui/needs-you.fixture"

function mount(needs = sampleNeeds) {
  const onOpenQueue = vi.fn()
  render(<NeedsYouPanel needs={needs} onOpenQueue={onOpenQueue} />)
  return onOpenQueue
}

describe("NeedsYouPanel", () => {
  it("opens the queue from a proposal", () => {
    const onOpenQueue = mount()

    fireEvent.click(screen.getByRole("button", { name: /SEL-54/ }))

    expect(onOpenQueue).toHaveBeenCalledOnce()
  })

  it("says so when nothing needs the user", () => {
    mount(nothingNeeded)

    expect(screen.getByText(/Nothing needs you now/)).toBeInTheDocument()
    expect(screen.queryByRole("region")).not.toBeInTheDocument()
  })
})
