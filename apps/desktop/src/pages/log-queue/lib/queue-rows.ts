import type { AppRef } from "@/shared/api/generated/AppRef"
import type { DaySummary } from "@/shared/api/generated/DaySummary"
import type { QueueItem } from "@/shared/api/generated/QueueItem"

import type { ProposalState } from "@/shared/api/generated/ProposalState"

export type QueueState = ProposalState["kind"]
export type QueueFilter = "all" | QueueState

export type QueueRow =
  | {
      kind: "group"
      state: QueueState
      title: string
      count: number
      minutes: number
    }
  | {
      kind: "day"
      group: string
      date: string
      /** This group's proposals on that day. */
      ids: number[]
      /** Their minutes together. */
      minutes: number
      summary: DaySummary | undefined
    }
  | {
      kind: "item"
      item: QueueItem
      /** The day row this proposal sits under. */
      day: string
    }

/** The queue's sections, in the order the user acts on them. */
const STATES: readonly [QueueState, string][] = [
  ["proposed", "To approve"],
  ["approved", "Approved · not logged yet"],
  ["logged", "Logged"],
  ["rejected", "Rejected"],
]

const minutesOf = (items: readonly QueueItem[]) =>
  items.reduce((sum, item) => sum + item.proposal.minutes, 0)

/**
 * A section row per state, then a row per day, newest first, with the
 * day's minutes and its totals when known, then that day's proposals.
 */
export function queueRows(
  items: readonly QueueItem[],
  filter: QueueFilter,
  summaries: ReadonlyMap<string, DaySummary> = new Map(),
) {
  const rows: QueueRow[] = []
  for (const [state, title] of STATES) {
    if (filter !== "all" && filter !== state) continue
    const members = items.filter((item) => item.proposal.state.kind === state)
    if (members.length === 0) continue
    rows.push({
      kind: "group",
      state,
      title,
      count: members.length,
      minutes: minutesOf(members),
    })
    const byDay = new Map<string, QueueItem[]>()
    for (const item of members) {
      byDay.set(item.proposal.date, [
        ...(byDay.get(item.proposal.date) ?? []),
        item,
      ])
    }
    for (const date of [...byDay.keys()].sort().reverse()) {
      const day = byDay.get(date) ?? []
      const dayKey = `${state}:${date}`
      rows.push({
        kind: "day",
        group: dayKey,
        date,
        ids: day.map((item) => item.proposal.id),
        minutes: minutesOf(day),
        summary: summaries.get(date),
      })
      for (const item of day) rows.push({ kind: "item", item, day: dayKey })
    }
  }
  return rows
}

/** The days that have proposals waiting or approved. */
export function openDates(items: readonly QueueItem[]) {
  return [
    ...new Set(
      items
        .filter((item) =>
          ["proposed", "approved"].includes(item.proposal.state.kind),
        )
        .map((item) => item.proposal.date),
    ),
  ].sort()
}

export function filterCounts(items: readonly QueueItem[]) {
  const count = (state: QueueState) =>
    items.filter((item) => item.proposal.state.kind === state).length
  return {
    all: items.length,
    proposed: count("proposed"),
    approved: count("approved"),
    logged: count("logged"),
    rejected: count("rejected"),
  } satisfies Record<QueueFilter, number>
}

/** The app whose icon stands for the agent that proposed a worklog. */
export function proposerApp(agent: string): AppRef {
  if (/claude/i.test(agent))
    return { id: "com.anthropic.claudefordesktop", name: agent }
  if (/codex/i.test(agent)) return { id: "com.openai.codex", name: agent }
  return { id: `agent:${agent}`, name: agent }
}
