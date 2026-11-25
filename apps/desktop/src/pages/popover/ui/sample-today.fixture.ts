import type { DayTimeline } from "@/shared/api/generated/DayTimeline"

const minutes = (count: number) => count * 60_000
const item = (id: string, name: string, count: number) => ({
  app: { id, name },
  title: { kind: "known" as const, text: name },
  url: null,
  durationMs: minutes(count),
})

export const sampleToday: DayTimeline = {
  fromMs: 0,
  toMs: 24 * 3_600_000,
  blocks: [
    {
      kind: "apps",
      startMs: 0,
      endMs: minutes(479),
      dominant: { id: "com.jetbrains.rider", name: "Rider" },
      title: { kind: "known", text: "" },
      items: [
        item("com.jetbrains.rider", "Rider", 171),
        item("com.openai.codex", "Codex", 74),
        item("company.thebrowser.dia", "Dia", 73),
        item("com.microsoft.teams2", "Microsoft Teams", 58),
        item("com.anthropic.claudefordesktop", "Claude", 51),
        item("com.tinyspeck.slackmacgap", "Slack", 28),
        item("com.microsoft.Outlook", "Outlook", 22),
      ],
    },
  ],
  agents: [],
  work: [],
  commits: [],
  totals: {
    activeMs: minutes(477),
    awayMs: 0,
    agentMs: 0,
    split: {
      workMs: minutes(477),
      agentMs: 0,
      uncountedMs: 0,
    },
  },
}
