import { describe, expect, it } from "vitest"
import { windowTitle } from "@/entities/activity/lib/window-title"

describe("windowTitle", () => {
  it("never shows an empty title", () => {
    expect(windowTitle({ kind: "known", text: "  " })).toBe("Untitled window")
    expect(
      windowTitle({ kind: "unavailable", reason: "permissionDenied" }),
    ).toBe("Title hidden (Accessibility not allowed)")
    expect(windowTitle({ kind: "unavailable", reason: "noWindow" })).toBe(
      "No window",
    )
  })
})
