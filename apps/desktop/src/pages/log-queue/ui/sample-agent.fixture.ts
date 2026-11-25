import type { AgentRun } from "@/shared/api/generated/AgentRun"
import type { AgentSettings } from "@/shared/api/generated/AgentSettings"

const at = (day: number, hours: number, minutes = 0) =>
  new Date(2026, 9, day, hours, minutes).getTime()

export const sampleAgentSettings: AgentSettings = {
  agent: "claudeCode",
  customCommand: "",
  daily: true,
  dailyTime: "08:00",
  defaultRange: "sinceLastRun",
  lookupTools: "youtrack/get_issue,youtrack/search_issues",
  claudeHome: "",
  codexHome: "",
  instructions: "",
}

export const doneRun: AgentRun = {
  id: 2,
  agent: "claudeCode",
  fromMs: at(14, 0),
  toMs: at(16, 0),
  startedMs: at(16, 8),
  state: {
    kind: "done",
    finishedMs: at(16, 8, 3),
    summary:
      "Made 3 proposals. 4 spans from 14 Oct to 16 Oct need you because their issue is only a candidate: SEL-77 on 14 Oct 09:15–10:15, PD-275 on 15 Oct 13:00–14:00, SEL-82 on 15 Oct 14:00–14:45 and PD-282 on 16 Oct 09:15–09:45.",
  },
}

export const runningRun: AgentRun = {
  ...doneRun,
  id: 3,
  fromMs: at(15, 0),
  toMs: at(16, 10),
  startedMs: at(16, 10),
  state: { kind: "running" },
}

export const failedRun: AgentRun = {
  ...doneRun,
  id: 4,
  state: {
    kind: "failed",
    finishedMs: at(16, 12),
    error: "Cannot start claude: No such file or directory",
  },
}
