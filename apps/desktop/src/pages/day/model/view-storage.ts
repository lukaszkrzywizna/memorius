import { z } from "zod"
import {
  TIMELINE_VIEWS,
  type TimelineView,
} from "@/pages/day/lib/timeline-view"

const KEY = "memorius.day.view"
const storedViewSchema = z.enum(TIMELINE_VIEWS)

/** The timeline direction used last, so the Day view opens the way the user left it. */
export function readStoredView(): TimelineView | undefined {
  try {
    return storedViewSchema.safeParse(localStorage.getItem(KEY)).data
  } catch (error) {
    console.warn("Memorius could not read the stored timeline view", error)
    return undefined
  }
}

export function storeView(view: TimelineView) {
  try {
    localStorage.setItem(KEY, view)
  } catch (error) {
    // The view then lasts for this visit only.
    console.warn("Memorius could not store the timeline view", error)
  }
}
