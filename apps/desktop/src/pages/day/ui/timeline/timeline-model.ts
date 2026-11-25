import type { DayTimeline } from "@/shared/api/generated/DayTimeline"
import { windowTitle } from "@/entities/activity"
import { agentApp } from "@/pages/day/lib/agent-lanes"
import { formatClock } from "@/shared/lib/local-date"

type Lane = "apps" | "agents" | "work"
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
    return { apps: () => false, agents: () => false, work: () => false }
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
  }
}

/** The start of the selected item, so the view can scroll to it. */
export function selectionStartMs(
  timeline: DayTimeline,
  selection: Selection | null,
) {
  if (!selection) return undefined
  switch (selection.lane) {
    case "apps":
      return timeline.blocks[selection.index]?.startMs
    case "agents":
      return timeline.agents[selection.index]?.startMs
    case "work":
      return timeline.work[selection.index]?.startMs
  }
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
  }
}
