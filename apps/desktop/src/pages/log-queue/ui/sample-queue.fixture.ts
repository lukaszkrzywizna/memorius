import type { Proposal } from "@/shared/api/generated/Proposal"
import type { QueueItem } from "@/shared/api/generated/QueueItem"
import { todayIso } from "@/shared/lib/local-date"

const at = (day: number, hours: number, minutes = 0) =>
  new Date(2026, 9, day, hours, minutes).getTime()
const minutes = (count: number) => count * 60_000

const proposal = (
  fields: Partial<Proposal> &
    Pick<Proposal, "id" | "issue" | "startMs" | "endMs" | "minutes" | "text">,
): Proposal => ({
  workType: "Development",
  agent: "Claude Code",
  createdMs: at(9, 17, 42),
  state: { kind: "proposed" },
  lastError: null,
  revision: 1,
  date: todayIso(new Date(fields.startMs)),
  ...fields,
})

export const sampleQueue: QueueItem[] = [
  {
    proposal: proposal({
      id: 1,
      issue: "SEL-54",
      startMs: at(9, 13, 31),
      endMs: at(9, 14, 50),
      minutes: 79,
      text: "Moved the customer history dashboard to the app stack and checked the new Container App revision.",
    }),
    apps: [
      {
        app: { id: "com.anthropic.claudefordesktop", name: "Claude" },
        durationMs: minutes(51),
      },
      {
        app: { id: "company.thebrowser.dia", name: "Dia" },
        durationMs: minutes(28),
      },
    ],
  },
  {
    proposal: proposal({
      id: 2,
      issue: "IS-61",
      startMs: at(7, 16, 10),
      endMs: at(7, 16, 50),
      minutes: 40,
      workType: "Review",
      agent: "Codex",
      text: "Compared the north and south store results against production for the shared catalog SQL.",
    }),
    apps: [
      {
        app: { id: "com.jetbrains.rider", name: "Rider" },
        durationMs: minutes(28),
      },
      {
        app: { id: "company.thebrowser.dia", name: "Dia" },
        durationMs: minutes(12),
      },
    ],
  },
  {
    proposal: proposal({
      id: 3,
      issue: "INT-177",
      startMs: at(9, 11, 30),
      endMs: at(9, 12, 44),
      minutes: 74,
      agent: "Codex",
      text: "Job runner migration batch 2: order and stock syncs.",
      state: { kind: "approved", atMs: at(9, 18) },
      lastError:
        "YouTrack answered 403: no permission to add work items to INT-177.",
    }),
    apps: [
      {
        app: { id: "com.openai.codex", name: "Codex" },
        durationMs: minutes(74),
      },
    ],
  },
  {
    proposal: proposal({
      id: 4,
      issue: "INT-290",
      startMs: at(9, 9, 8),
      endMs: at(9, 11),
      minutes: 112,
      text: "Skip unchanged customers and prices in the order sync.",
      state: { kind: "logged", atMs: at(9, 18, 5), workItemId: "2-48211" },
    }),
    apps: [
      {
        app: { id: "com.jetbrains.rider", name: "Rider" },
        durationMs: minutes(112),
      },
    ],
  },
]
