import { describe, expect, it } from "vitest"
import { formatDuration } from "@/shared/lib/duration"

describe("formatDuration", () => {
  it.each([
    [0, "<1m"],
    [59_000, "<1m"],
    [60_000, "1m"],
    [47 * 60_000, "47m"],
    [120 * 60_000, "2h"],
    [479 * 60_000, "7h 59m"],
  ])("formats %i ms as %s", (ms, text) => {
    expect(formatDuration(ms)).toBe(text)
  })
})
