import type { LaneName } from "@/pages/day/lib/lane-widths"

/** Room for the lane names left of the timeline. */
export const LABEL_PX = 96
/** One agent row: readable icon and title at any number of parallel sessions. */
const AGENT_ROW_PX = 40
const AGENT_MIN_PX = 88

/** A folded lane's height: one row of marks, more for parallel sessions. */
const COLLAPSED_PX = 20
const COLLAPSED_TRACK_PX = 12
const COMMITS_PX = 36

/** Vertical positions of the horizontal timeline's lanes. */
export function horizontalLanes(
  parallelSessions: number,
  collapsed: ReadonlySet<LaneName> = new Set(),
  /** Whether the Commits lane shows (only with commits to show). */
  withCommits = false,
) {
  const tall = (lane: LaneName, height: number, tracks = 1) =>
    collapsed.has(lane)
      ? Math.max(COLLAPSED_PX, tracks * COLLAPSED_TRACK_PX + 8)
      : height
  const agentsHeight = tall(
    "agents",
    Math.max(AGENT_MIN_PX, parallelSessions * AGENT_ROW_PX),
    parallelSessions,
  )
  const apps = { top: 32, height: tall("apps", 80) }
  const agents = { top: apps.top + apps.height + 16, height: agentsHeight }
  const commits = {
    top: agents.top + agentsHeight + (withCommits ? 16 : 0),
    height: withCommits ? COMMITS_PX : 0,
  }
  const work = {
    top: commits.top + commits.height + 16,
    height: tall("work", 50),
  }
  const logged = {
    top: work.top + work.height + 16,
    height: tall("logged", 50),
  }
  const prompts = { top: logged.top + logged.height + 16, height: 36 }
  const card = prompts.top + prompts.height + 22
  return {
    apps,
    agents,
    commits,
    work,
    logged,
    prompts,
    /** Where the grid lines end and the details card starts. */
    linesHeight: prompts.top + prompts.height - 28,
    card,
  }
}
