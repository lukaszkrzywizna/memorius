import type { AgentBlock } from "@/shared/api/generated/AgentBlock"
import type { DayTimeline } from "@/shared/api/generated/DayTimeline"
import type { Evidence } from "@/shared/api/generated/Evidence"
import { windowTitle } from "@/entities/activity"
import { agentApp } from "@/pages/day/lib/agent-lanes"
import { ruleText } from "@/pages/day/lib/work-lane"
import type { Selection } from "@/pages/day/ui/timeline/timeline-model"
import {
  type DetailRow,
  DetailsCard,
  type DetailsCardProps,
} from "@/pages/day/ui/details/details-card"
import type { SessionsPrivateRequest } from "@/shared/api/memorius-commands"
import { formatDuration } from "@/shared/lib/duration"
import { formatClock } from "@/shared/lib/local-date"

type SelectionDetailsProps = {
  timeline: DayTimeline
  selection: Selection
  onMarkPrivate: (request: SessionsPrivateRequest) => void
  onClose: () => void
}

const range = (item: { startMs: number; endMs: number }) =>
  `${formatClock(item.startMs)}–${formatClock(item.endMs)}`

/** The details card for whatever the user selected on the timeline. */
export function SelectionDetails({
  timeline,
  selection,
  onMarkPrivate,
  onClose,
}: SelectionDetailsProps) {
  const props = detailsFor(timeline, selection, onMarkPrivate)
  return props ? <DetailsCard {...props} onClose={onClose} /> : null
}

function detailsFor(
  timeline: DayTimeline,
  selection: Selection,
  onMarkPrivate: SelectionDetailsProps["onMarkPrivate"],
): Omit<DetailsCardProps, "onClose"> | undefined {
  switch (selection.lane) {
    case "apps": {
      const block = timeline.blocks[selection.index]
      if (block?.kind !== "apps") return undefined
      const longest = Math.max(1, ...block.items.map((item) => item.durationMs))
      return {
        label: "Selected activity",
        app: block.dominant,
        title: windowTitle(block.title),
        sub: `${block.dominant.name} · ${range(block)}`,
        chips: [],
        durationMs: block.endMs - block.startMs,
        rows: block.items.map((item) => ({
          label: "",
          app: item.app,
          text: item.url
            ? `${windowTitle(item.title)} · ${item.url}`
            : windowTitle(item.title),
          value: formatDuration(item.durationMs),
          share: Math.round((item.durationMs / longest) * 100),
        })),
      }
    }
    case "agents": {
      const agent = timeline.agents[selection.index]
      if (!agent) return undefined
      const app = agentApp(agent.source)
      const title = agent.title ?? agent.branch ?? app.name
      return {
        label: "Agent session",
        app,
        title: agent.issue ? `${agent.issue} · ${title}` : title,
        sub: `${app.name} · ${range(agent)}`,
        chips: [
          {
            text: `${agent.prompts} prompt${agent.prompts === 1 ? "" : "s"}`,
          },
          ...(agent.private ? [{ text: "Private" }] : []),
        ],
        durationMs: agent.endMs - agent.startMs,
        rows: [
          ...(agent.branch ? [{ label: "Branch", text: agent.branch }] : []),
          ...agent.commits.map((subject) => ({
            label: "Commit",
            text: subject,
          })),
          ...(agent.split.agentMs > 0
            ? [
                {
                  label: "Counted",
                  text: "Up to 30 min after your prompt, while you were away",
                  value: formatDuration(agent.split.agentMs),
                },
              ]
            : []),
          ...(agent.split.uncountedMs > 0
            ? [
                {
                  label: "Not counted",
                  text: agent.private
                    ? "Private session"
                    : "No prompt from you in the 30 min before",
                  value: formatDuration(agent.split.uncountedMs),
                },
              ]
            : []),
        ],
        privacy: privacyOf(agent, onMarkPrivate),
      }
    }
    case "work": {
      const span = timeline.work[selection.index]
      if (!span) return undefined
      const candidate = span.confidence === "medium"
      return {
        label: "Detected work",
        title: candidate ? `${span.issue} · candidate` : span.issue,
        sub: `Work context · ${range(span)}`,
        chips: [{ text: candidate ? "Medium confidence" : "High confidence" }],
        durationMs: span.endMs - span.startMs,
        rows: [
          ...span.evidence.map(evidenceRow),
          ...(span.candidates.length > 0
            ? [
                {
                  label: "Also",
                  text: span.candidates
                    .map((other) => `${other.issue} ${other.support}%`)
                    .join(", "),
                },
              ]
            : []),
        ],
        sessions: timeline.agents
          .filter(
            (agent) => agent.startMs < span.endMs && span.startMs < agent.endMs,
          )
          .map((agent) => {
            const app = agentApp(agent.source)
            return {
              key: `${agent.startMs}:${agent.sessions.map((s) => s.sessionId).join(",")}`,
              app,
              title: agent.issue
                ? `${agent.issue} · ${agent.title ?? agent.branch ?? app.name}`
                : (agent.title ?? agent.branch ?? app.name),
              note: sessionNote(agent),
              ...privacyOf(agent, onMarkPrivate),
            }
          }),
      }
    }
  }
}

function privacyOf(
  agent: AgentBlock,
  onMarkPrivate: SelectionDetailsProps["onMarkPrivate"],
) {
  return {
    private: agent.private,
    onPrivateChange: (hidden: boolean) =>
      onMarkPrivate({ sessions: agent.sessions, private: hidden }),
  }
}

function sessionNote(agent: AgentBlock) {
  if (agent.private) return "Private · not counted"
  const parts = [
    agent.split.agentMs > 0 &&
      `${formatDuration(agent.split.agentMs)} counted while you were away`,
    agent.split.uncountedMs > 0 &&
      `${formatDuration(agent.split.uncountedMs)} not counted`,
  ].filter(Boolean)
  return parts.length > 0 ? parts.join(" · ") : "Ran while you worked"
}

function evidenceRow(evidence: Evidence): DetailRow {
  switch (evidence.kind) {
    case "windowTitle":
      return { label: "Window", text: `${evidence.app} · ${evidence.title}` }
    case "url":
      return { label: "Address", text: `${evidence.app} · ${evidence.url}` }
    case "session":
      return {
        label: evidence.source === "claudeCode" ? "Claude Code" : "Codex",
        text: [
          evidence.title,
          evidence.branch && `branch ${evidence.branch}`,
          evidence.prompts > 0
            ? `${evidence.prompts} prompt${evidence.prompts === 1 ? "" : "s"} here`
            : "ran in the background",
        ]
          .filter(Boolean)
          .join(" · "),
      }
    case "commit":
      return { label: "Commit", text: evidence.subject }
    case "rule":
      return { label: "Rule", text: ruleText(evidence.rule) }
  }
}
