import type { DayTimeline } from "@/shared/api/generated/DayTimeline"
import { windowTitle } from "@/entities/activity"
import { agentApp } from "@/pages/day/lib/agent-lanes"
import { formatDuration } from "@/shared/lib/duration"
import { formatClock } from "@/shared/lib/local-date"

type Lane = "apps" | "agents" | "work" | "logged"
export type Selection = { lane: Lane; index: number }

/** Every prompt the user sent today, with the session it went to. */
export function promptMarks(timeline: DayTimeline) {
  return timeline.agents
    .flatMap((agent) => {
      const app = agentApp(agent.source)
      const session = agent.issue
        ? `${agent.issue} · ${agent.title ?? app.name}`
        : (agent.title ?? agent.branch ?? app.name)
      return agent.promptTimesMs.map((atMs) => ({
        atMs,
        app,
        session,
        sessions: agent.sessions,
        label: `Prompt at ${formatClock(atMs)}, ${app.name}, ${session}`,
      }))
    })
    .sort((a, b) => a.atMs - b.atMs)
}

export type PromptMark = ReturnType<typeof promptMarks>[number]

/** With an issue highlighted, everything else steps back. */
export function dimmedBy(timeline: DayTimeline, highlight: string | null) {
  if (!highlight)
    return {
      apps: () => false,
      agents: () => false,
      work: () => false,
      logged: () => false,
    }
  const runs = timeline.work.filter((span) => span.issue === highlight)
  return {
    apps: (index: number) => {
      const block = timeline.blocks[index]
      return !runs.some(
        (span) =>
          block && span.startMs < block.endMs && block.startMs < span.endMs,
      )
    },
    agents: (index: number) => timeline.agents[index]?.issue !== highlight,
    work: (index: number) => timeline.work[index]?.issue !== highlight,
    logged: (index: number) =>
      timeline.logged.blocks[index]?.issue !== highlight,
  }
}

/** Logged time against the day's work, for the YouTrack lane's name. */
export function loggedSummary(timeline: DayTimeline) {
  const loggedMs = timeline.logged.blocks
    .filter((block) => block.state === "logged")
    .reduce((sum, block) => sum + block.endMs - block.startMs, 0)
  const logged = loggedMs > 0 ? formatDuration(loggedMs) : "0m"
  return `${logged} / ${formatDuration(timeline.totals.split.workMs)}`
}

/** The selected item, for placing its details card. */
export function selectedItem(timeline: DayTimeline, selection: Selection) {
  switch (selection.lane) {
    case "apps":
      return timeline.blocks[selection.index]
    case "agents":
      return timeline.agents[selection.index]
    case "work":
      return timeline.work[selection.index]
    case "logged":
      return timeline.logged.blocks[selection.index]
  }
}

/** The start of the selected item, so the view can scroll to it. */
export function selectionStartMs(
  timeline: DayTimeline,
  selection: Selection | null,
) {
  return selection ? selectedItem(timeline, selection)?.startMs : undefined
}

/** A lane's blocks with what each is and when, for a folded lane's marks. */
export function laneEntries(timeline: DayTimeline, lane: Lane) {
  const when = (span: { startMs: number; endMs: number }) =>
    `${formatClock(span.startMs)}–${formatClock(span.endMs)}`
  switch (lane) {
    case "apps":
      return timeline.blocks.flatMap((block, index) =>
        block.kind === "apps"
          ? [
              {
                index,
                span: block,
                label: `${block.dominant.name} · ${windowTitle(block.title)} · ${when(block)}`,
              },
            ]
          : [],
      )
    case "agents":
      return timeline.agents.map((agent, index) => ({
        index,
        span: agent,
        label: `${agent.issue ?? agent.title ?? agentApp(agent.source).name} · ${when(agent)}`,
      }))
    case "work":
      return timeline.work.map((span, index) => ({
        index,
        span,
        label: `${span.issue} · ${when(span)}`,
      }))
    case "logged":
      return timeline.logged.blocks.map((block, index) => ({
        index,
        span: block,
        label: `${block.issue} · ${when(block)}`,
      }))
  }
}
