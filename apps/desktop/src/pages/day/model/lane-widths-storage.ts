import { z } from "zod"
import {
  LANE_NAMES,
  type LaneName,
  type LaneWidths,
} from "@/pages/day/lib/lane-widths"

const KEY = "memorius.day.laneWidths"
const storedSchema = z.object({
  apps: z.number().positive().optional(),
  agents: z.number().positive().optional(),
})

/** The lane widths the user dragged to, so the Day view keeps them. */
export function readStoredLaneWidths(): LaneWidths {
  try {
    const parsed = storedSchema.safeParse(
      JSON.parse(localStorage.getItem(KEY) ?? "{}"),
    )
    return parsed.success ? parsed.data : {}
  } catch (error) {
    console.warn("Memorius could not read the stored lane widths", error)
    return {}
  }
}

export function storeLaneWidths(widths: LaneWidths) {
  try {
    localStorage.setItem(KEY, JSON.stringify(widths))
  } catch (error) {
    // The widths then last for this visit only.
    console.warn("Memorius could not store the lane widths", error)
  }
}

const COLLAPSED_KEY = "memorius.day.collapsedLanes"
const collapsedSchema = z.array(z.enum(LANE_NAMES))

/** The lanes the user folded, so the Day view keeps them folded. */
export function readStoredCollapsedLanes(): LaneName[] {
  try {
    const parsed = collapsedSchema.safeParse(
      JSON.parse(localStorage.getItem(COLLAPSED_KEY) ?? "[]"),
    )
    return parsed.success ? parsed.data : []
  } catch (error) {
    console.warn("Memorius could not read the folded lanes", error)
    return []
  }
}

export function storeCollapsedLanes(lanes: ReadonlySet<LaneName>) {
  try {
    localStorage.setItem(COLLAPSED_KEY, JSON.stringify([...lanes]))
  } catch (error) {
    // The lanes then stay folded for this visit only.
    console.warn("Memorius could not store the folded lanes", error)
  }
}
