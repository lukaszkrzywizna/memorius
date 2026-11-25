/** Widths the user dragged the vertical timeline's lanes to, in pixels. */
export type LaneWidths = { apps?: number; agents?: number }

export type ResizableLane = keyof LaneWidths

/** A lane that can be folded to a thin strip of marks. */
export type LaneName = "apps" | "agents" | "work"
export const LANE_NAMES = ["apps", "agents", "work"] as const

/**
 * A folded lane keeps the room its name needs in the vertical view (the
 * header and the lanes are separate grids, so it is a fixed width).
 */
const COLLAPSED_PX: Record<LaneName, number> = {
  apps: 64,
  agents: 80,
  work: 64,
}

const APPS_MIN_PX = 200
const WORK_MIN_PX = 150
const COMMITS_MIN_PX = 160
/** Each parallel agent session keeps at least this width by default. */
const AGENT_COLUMN_PX = 96
const AGENTS_MIN_PX = 160
/** The narrowest a dragged lane may get, so its blocks stay readable. */
const LANE_MIN_PX = 120
const LANE_MAX_PX = 2400

export function clampLane(width: number) {
  return Math.round(Math.min(LANE_MAX_PX, Math.max(LANE_MIN_PX, width)))
}

/**
 * The grid columns: time, Apps, Agents, Work, prompts. A lane the user
 * dragged keeps that width; the others share the room above a minimum, and
 * Work takes what is left, so Apps and Work never collapse when many agent
 * sessions run at once.
 */
export function laneColumns(
  widths: LaneWidths,
  parallel: number,
  collapsed: ReadonlySet<LaneName> = new Set(),
  withCommits = false,
) {
  const apps = collapsed.has("apps")
    ? `${COLLAPSED_PX.apps}px`
    : widths.apps
      ? `${widths.apps}px`
      : `minmax(${APPS_MIN_PX}px, 1.3fr)`
  const agents = collapsed.has("agents")
    ? `${COLLAPSED_PX.agents}px`
    : widths.agents
      ? `${widths.agents}px`
      : `minmax(${Math.max(AGENTS_MIN_PX, parallel * AGENT_COLUMN_PX)}px, 1.25fr)`
  const work = collapsed.has("work")
    ? `${COLLAPSED_PX.work}px`
    : `minmax(${WORK_MIN_PX}px, 1fr)`
  const commits = withCommits ? ` minmax(${COMMITS_MIN_PX}px, 0.8fr)` : ""
  return `52px ${apps} ${agents}${commits} ${work} 30px`
}
