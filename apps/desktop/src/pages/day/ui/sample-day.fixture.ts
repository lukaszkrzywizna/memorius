import type { AppRef } from "@/shared/api/generated/AppRef"
import type { Block } from "@/shared/api/generated/Block"
import type { AgentBlock } from "@/shared/api/generated/AgentBlock"
import type { DayCommit } from "@/shared/api/generated/DayCommit"
import type { DayTimeline } from "@/shared/api/generated/DayTimeline"
import type { TimeSplit } from "@/shared/api/generated/TimeSplit"
import type { WorkSpan } from "@/shared/api/generated/WorkSpan"
import { parseIsoDate } from "@/shared/lib/local-date"

/** Friday 9 October 2026, shaped like the Day timeline artboard. */
export const sampleDate = "2026-10-09"

const dayStart = parseIsoDate(sampleDate).getTime()
const at = (clock: string) => {
  const [hours = 0, minutes = 0] = clock.split(":").map(Number)
  return dayStart + (hours * 60 + minutes) * 60_000
}
const app = (id: string, name: string): AppRef => ({ id, name })
const rider = app("com.jetbrains.rider", "Rider")
const codex = app("com.openai.codex", "Codex")
const dia = app("company.thebrowser.dia", "Dia")
const slack = app("com.tinyspeck.slackmacgap", "Slack")
const teams = app("com.microsoft.teams2", "Microsoft Teams")
const minutes = (count: number) => count * 60_000
/** Minutes of the user's work, agent time counted in it, and not counted. */
const split = (work: number, agent = 0, uncounted = 0): TimeSplit => ({
  workMs: minutes(work),
  agentMs: minutes(agent),
  uncountedMs: minutes(uncounted),
})
const title = (text: string) => ({ kind: "known" as const, text })
const item = (owner: AppRef, text: string, count: number) => ({
  app: owner,
  title: title(text),
  url: null,
  durationMs: minutes(count),
})

const blocks: Block[] = [
  {
    kind: "apps",
    startMs: at("08:45"),
    endMs: at("09:15"),
    dominant: slack,
    title: title("#integration-alerts"),
    items: [item(slack, "#integration-alerts", 9), item(slack, "#dev", 5)],
  },
  {
    kind: "apps",
    startMs: at("09:15"),
    endMs: at("10:30"),
    dominant: rider,
    title: title("OrderSync.fs — Shop"),
    items: [
      item(rider, "OrderSync.fs — Shop", 52),
      item(rider, "OrderSyncTests.fs — Shop", 24),
      item(dia, "Pull request #1814 · GitHub", 12),
      item(slack, "#dev", 2),
    ],
  },
  {
    kind: "apps",
    startMs: at("11:00"),
    endMs: at("11:30"),
    dominant: teams,
    title: title("Daily stand-up"),
    items: [item(teams, "Daily stand-up", 28)],
  },
  {
    kind: "apps",
    startMs: at("11:30"),
    endMs: at("12:45"),
    dominant: codex,
    title: title("INT-177 Job runner migration batch 2"),
    items: [
      item(codex, "INT-177 Job runner migration batch 2", 61),
      item(dia, "Codex docs", 13),
    ],
  },
  { kind: "away", startMs: at("12:45"), endMs: at("13:30") },
  {
    kind: "apps",
    startMs: at("13:30"),
    endMs: at("16:30"),
    dominant: rider,
    title: title("CatalogSource.fs — Shop"),
    items: [
      item(rider, "CatalogSource.fs — Shop", 120),
      item(dia, "IS-61 · YouTrack", 40),
      item(slack, "#dev", 30),
    ],
  },
  { kind: "locked", startMs: at("16:30"), endMs: at("16:45") },
]

/** Agent sessions; parallel ones share the lane, and agents keep working while the user is away. */
const agents: AgentBlock[] = [
  {
    startMs: at("08:45"),
    endMs: at("09:15"),
    source: "claudeCode",
    title: "Customer history dashboard",
    branch: "claude/sel-54-order-history",
    prompts: 0,
    promptTimesMs: [],
    issue: "SEL-54",
    commits: [],
    sessions: [{ source: "claudeCode", sessionId: "sel54" }],
    private: false,
    split: split(30),
  },
  {
    startMs: at("09:00"),
    endMs: at("10:30"),
    source: "claudeCode",
    title: "Order sync re-sends unchanged rows",
    branch: "claude/int-290-order-sync-unchanged",
    prompts: 3,
    promptTimesMs: [at("09:05"), at("09:20"), at("09:47")],
    issue: "INT-290",
    commits: ["[INT-290] Skip unchanged employees"],
    sessions: [{ source: "claudeCode", sessionId: "int290" }],
    private: false,
    split: split(90),
  },
  {
    startMs: at("11:15"),
    endMs: at("12:45"),
    source: "codex",
    title: "INT-177 Job runner migration batch 2",
    branch: null,
    prompts: 4,
    promptTimesMs: [at("11:18"), at("11:22"), at("11:40"), at("12:05")],
    issue: "INT-177",
    commits: [],
    sessions: [{ source: "codex", sessionId: "int177" }],
    private: false,
    split: split(90),
  },
  {
    startMs: at("12:30"),
    endMs: at("14:00"),
    source: "claudeCode",
    title: "Order Search readiness",
    branch: "claude/is61-uuid-keys-pg",
    prompts: 5,
    promptTimesMs: [
      at("12:31"),
      at("12:52"),
      at("13:18"),
      at("13:40"),
      at("13:55"),
    ],
    issue: "IS-61",
    commits: ["[IS-61] UUID keys on PostgreSQL"],
    sessions: [{ source: "claudeCode", sessionId: "is61" }],
    private: false,
    split: split(45, 45),
  },
]

/** Work spans follow the issue, not the app blocks. */
const work: WorkSpan[] = [
  {
    startMs: at("08:45"),
    endMs: at("09:00"),
    issue: "SEL-54",
    confidence: "medium",
    support: 30,
    candidates: [],
    split: split(6),
    evidence: [
      {
        kind: "session",
        source: "claudeCode",
        title: "Customer history dashboard",
        branch: "claude/sel-54-order-history",
        prompts: 0,
      },
    ],
  },
  {
    startMs: at("09:00"),
    endMs: at("10:15"),
    issue: "INT-290",
    confidence: "high",
    support: 80,
    candidates: [],
    split: split(70),
    evidence: [
      {
        kind: "session",
        source: "claudeCode",
        title: "Order sync re-sends unchanged rows",
        branch: "claude/int-290-order-sync-unchanged",
        prompts: 3,
      },
      { kind: "commit", subject: "[INT-290] Skip unchanged employees" },
    ],
  },
  {
    startMs: at("11:30"),
    endMs: at("12:30"),
    issue: "INT-177",
    confidence: "high",
    support: 75,
    candidates: [],
    split: split(55),
    evidence: [
      {
        kind: "session",
        source: "codex",
        title: "INT-177 Job runner migration batch 2",
        branch: null,
        prompts: 4,
      },
    ],
  },
  {
    startMs: at("12:45"),
    endMs: at("13:30"),
    issue: "IS-61",
    confidence: "medium",
    support: 100,
    candidates: [],
    split: split(45, 45),
    evidence: [
      {
        kind: "session",
        source: "claudeCode",
        title: "Order Search readiness",
        branch: "claude/is61-uuid-keys-pg",
        prompts: 0,
      },
    ],
  },
  {
    startMs: at("13:30"),
    endMs: at("16:00"),
    issue: "IS-61",
    confidence: "high",
    support: 60,
    candidates: [],
    split: split(150),
    evidence: [{ kind: "windowTitle", app: "Dia", title: "IS-61 · YouTrack" }],
  },
]

const logged: DayTimeline["logged"] = {
  blocks: [
    {
      startMs: at("09:00"),
      endMs: at("10:15"),
      key: "proposal-1",
      issue: "INT-290",
      summary: "Manning push re-sends unchanged employees",
      minutes: 75,
      workType: "Development",
      text: "Skip unchanged employees in the push",
      state: "logged",
      estimated: false,
    },
    {
      startMs: at("11:30"),
      endMs: at("12:30"),
      key: "8-1234",
      issue: "INT-177",
      summary: "Sync migration batch 2",
      minutes: 60,
      workType: null,
      text: null,
      state: "logged",
      estimated: true,
    },
    {
      startMs: at("12:45"),
      endMs: at("14:00"),
      key: "proposal-2",
      issue: "IS-61",
      summary: null,
      minutes: 75,
      workType: "Development",
      text: "One Vista read model",
      state: "proposed",
      estimated: false,
    },
  ],
  gaps: [
    { startMs: at("10:15"), endMs: at("11:30") },
    { startMs: at("14:00"), endMs: at("16:00") },
  ],
}

export const sampleDay: DayTimeline = {
  fromMs: dayStart,
  toMs: dayStart + 24 * 3_600_000,
  blocks,
  agents,
  work,
  commits: [],
  logged,
  totals: {
    activeMs: minutes(415),
    awayMs: minutes(56),
    agentMs: minutes(270),
    split: split(430, 45),
  },
}

export const emptyDay: DayTimeline = {
  fromMs: dayStart,
  toMs: dayStart + 24 * 3_600_000,
  blocks: [],
  agents: [],
  work: [],
  commits: [],
  logged: { blocks: [], gaps: [] },
  totals: { activeMs: 0, awayMs: 0, agentMs: 0, split: split(0) },
}

/** A day with eight agent sessions running at once, for crowded lanes. */
export const busyDay: DayTimeline = {
  ...sampleDay,
  agents: Array.from({ length: 8 }, (_, index) => ({
    ...agents[index % agents.length]!,
    startMs: at("09:00") + index * 10 * 60_000,
    endMs: at("12:00") + index * 15 * 60_000,
    title: `Parallel session ${index + 1}`,
    sessions: [{ source: "claudeCode" as const, sessionId: `busy-${index}` }],
  })),
}

/** Commits in the session folders that day, for the Commits lane design. */
export const sampleCommits: DayCommit[] = [
  {
    hash: "00000010c0ffee",
    atMs: at("09:42"),
    subject: "[INT-290] Compare a hash of the normalized row",
    repo: "Shop",
    branch: "claude/int-290-push",
  },
  {
    hash: "00000020c0ffee",
    atMs: at("10:18"),
    subject: "[INT-290] Skip unchanged employees",
    repo: "Shop",
    branch: "claude/int-290-push",
  },
  {
    hash: "00000030c0ffee",
    atMs: at("10:24"),
    subject: "[INT-290] Cover cost codes in the push test",
    repo: "Shop",
    branch: "claude/int-290-push",
  },
  {
    hash: "00000040c0ffee",
    atMs: at("12:05"),
    subject: "[INT-177] Move the order sync to the job runner",
    repo: "Shop",
    branch: "codex/int-177-batch-2",
  },
  {
    hash: "00000050c0ffee",
    atMs: at("13:47"),
    subject: "[IS-61] UUID keys on PostgreSQL",
    repo: "Shop",
    branch: "claude/is61-uuid-keys-pg",
  },
  {
    hash: "00000060c0ffee",
    atMs: at("15:10"),
    subject: "Show a prompt's text on hover",
    repo: "memorius",
    branch: "graphite-redesign",
  },
]
