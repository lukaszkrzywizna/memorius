import { describe, expect, it } from "vitest"
import { clampLane, laneColumns } from "@/pages/day/lib/lane-widths"

describe("lane widths", () => {
  it("keeps Apps and Work from collapsing when many agents run at once", () => {
    expect(laneColumns({}, 12)).toBe(
      "52px minmax(200px, 1.3fr) minmax(1152px, 1.25fr) minmax(150px, 1fr) minmax(150px, 1fr) 30px",
    )
  })

  it("uses the widths the user dragged to, within limits", () => {
    expect(laneColumns({ apps: 320, agents: 480 }, 12)).toBe(
      "52px 320px 480px minmax(150px, 1fr) minmax(150px, 1fr) 30px",
    )
    expect(clampLane(40)).toBe(120)
  })

  it("adds the Commits lane between Agents and Work when there are commits", () => {
    expect(laneColumns({}, 1, new Set(), true)).toBe(
      "52px minmax(200px, 1.3fr) minmax(160px, 1.25fr) minmax(160px, 0.8fr) minmax(150px, 1fr) minmax(150px, 1fr) 30px",
    )
  })

  it("folds a lane to a thin strip", () => {
    expect(
      laneColumns({ apps: 320 }, 2, new Set(["apps", "work", "logged"])),
    ).toBe("52px 64px minmax(192px, 1.25fr) 64px 96px 30px")
  })
})
