import { z } from "zod"
import { ZOOM_MINUTES, type ZoomMinutes } from "@/pages/day/lib/zoom"

const KEY = "memorius.day.zoom"
const storedZoomSchema = z
  .string()
  .transform(Number)
  .pipe(z.literal(ZOOM_MINUTES))

/** The zoom used last, so the Day view opens the way the user left it. */
export function readStoredZoom(): ZoomMinutes | undefined {
  try {
    return storedZoomSchema.safeParse(localStorage.getItem(KEY)).data
  } catch (error) {
    console.warn("Memorius could not read the stored zoom", error)
    return undefined
  }
}

export function storeZoom(zoom: ZoomMinutes) {
  try {
    localStorage.setItem(KEY, String(zoom))
  } catch (error) {
    // The zoom then lasts for this visit only.
    console.warn("Memorius could not store the zoom", error)
  }
}
