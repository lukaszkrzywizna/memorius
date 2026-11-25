import { describe, expect, it } from "vitest"
import type { Block } from "@/shared/api/generated/Block"
import { topApps } from "@/pages/popover/lib/top-apps"

const app = (name: string) => ({ id: name, name })
const item = (name: string, minutes: number) => ({
  app: app(name),
  title: { kind: "known" as const, text: "" },
  url: null,
  durationMs: minutes * 60_000,
})

describe("topApps", () => {
  it("sums each app across blocks and sorts longest first", () => {
    const blocks: Block[] = [
      {
        kind: "apps",
        startMs: 0,
        endMs: 1,
        dominant: app("Rider"),
        title: { kind: "known", text: "" },
        items: [item("Rider", 30), item("Slack", 2)],
      },
      { kind: "away", startMs: 1, endMs: 2 },
      {
        kind: "apps",
        startMs: 2,
        endMs: 3,
        dominant: app("Slack"),
        title: { kind: "known", text: "" },
        items: [item("Slack", 40)],
      },
    ]

    expect(
      topApps(blocks).map((total) => [
        total.app.name,
        total.durationMs / 60_000,
      ]),
    ).toEqual([
      ["Slack", 42],
      ["Rider", 30],
    ])
  })
})
