import type { AgentKind } from "@/shared/api/generated/AgentKind"
import type { AgentRun } from "@/shared/api/generated/AgentRun"
import type { RunRange } from "@/shared/api/generated/RunRange"
import { formatClock, formatShortDate } from "@/shared/lib/local-date"

export const AGENT_NAMES: Record<AgentKind, string> = {
  claudeCode: "Claude Code",
  codex: "Codex",
  custom: "Custom command",
}

export const RANGE_LABELS: Record<RunRange, string> = {
  sinceLastRun: "Since the last run",
  day: "Last 24 hours",
  twoDays: "Last 48 hours",
  week: "Last week",
  month: "Last month",
}

/** `Wed 14 Oct–Fri 16 Oct`, the days a run covered. */
export function runDays(run: AgentRun) {
  const from = formatShortDate(run.fromMs)
  const to = formatShortDate(run.toMs - 1)
  return from === to ? from : `${from}–${to}`
}

/** One line about a run: what it does or did, and when. */
export function runText(run: AgentRun) {
  const name = AGENT_NAMES[run.agent]
  switch (run.state.kind) {
    case "running":
      return `${name} is preparing proposals for ${runDays(run)}…`
    case "done":
      return `${formatClock(run.state.finishedMs)} · ${run.state.summary || "Done"}`
    case "failed":
      return `${formatClock(run.state.finishedMs)} · Failed: ${run.state.error}`
  }
}
