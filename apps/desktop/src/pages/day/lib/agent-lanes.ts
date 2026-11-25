import type { AgentBlock } from "@/shared/api/generated/AgentBlock"
import type { AgentSource } from "@/shared/api/generated/AgentSource"
import type { AppRef } from "@/shared/api/generated/AppRef"

/** Where an agent block sits when sessions run in parallel: its sub-column and how many share the time. */
export type AgentPlace = { column: number; columns: number }

/**
 * Parallel sessions sit side by side, like overlapping calendar events:
 * each block takes the first free sub-column, and every block in a group
 * of overlapping blocks gets that group's number of sub-columns.
 */
export function agentPlaces(blocks: readonly AgentBlock[]): AgentPlace[] {
  const places: AgentPlace[] = []
  let group: number[] = []
  let groupEnd = Number.NEGATIVE_INFINITY
  let columnEnds: number[] = []
  const closeGroup = () => {
    for (const index of group) {
      const place = places[index]
      if (place) place.columns = columnEnds.length
    }
    group = []
    columnEnds = []
  }
  blocks.forEach((block, index) => {
    if (block.startMs >= groupEnd) closeGroup()
    let column = columnEnds.findIndex((end) => end <= block.startMs)
    if (column === -1) column = columnEnds.length
    columnEnds[column] = block.endMs
    groupEnd = Math.max(groupEnd, block.endMs)
    places[index] = { column, columns: 1 }
    group.push(index)
  })
  closeGroup()
  return places
}

/** The app whose icon stands for an agent. */
export function agentApp(source: AgentSource): AppRef {
  switch (source) {
    case "claudeCode":
      return { id: "com.anthropic.claudefordesktop", name: "Claude Code" }
    case "codex":
      return { id: "com.openai.codex", name: "Codex" }
  }
}
