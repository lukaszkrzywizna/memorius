import type { SearchResults } from "@/shared/api/generated/SearchResults"

const at = (day: number, hours: number, minutes = 0) =>
  new Date(2026, 9, day, hours, minutes).getTime()

/** IS-61 across a few days of week 41. */
export const sampleSearch: SearchResults = {
  issues: [
    {
      issue: "IS-61",
      worklogs: [
        { startMs: at(9, 13, 30), minutes: 205, logged: false },
        { startMs: at(8, 14), minutes: 95, logged: false },
        { startMs: at(7, 16, 10), minutes: 40, logged: true },
      ],
    },
  ],
  windows: [
    {
      app: { id: "company.thebrowser.dia", name: "Dia" },
      title: "IS-61 · YouTrack",
      url: "https://youtrack.example/issue/IS-61",
      totalMs: 68 * 60_000,
      spans: [
        [at(9, 15, 31), at(9, 16, 9)],
        [at(9, 10, 30), at(9, 11)],
      ],
    },
  ],
  sessions: [
    {
      source: "claudeCode",
      title: "Order Search readiness",
      branch: "claude/is61-uuid-keys-pg",
      startMs: at(9, 12, 30),
      endMs: at(9, 14),
    },
  ],
  truncated: false,
}
