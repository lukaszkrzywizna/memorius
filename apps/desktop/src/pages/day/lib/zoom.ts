/** Slot lengths the zoom offers; the core accepts these and more. */
export const ZOOM_MINUTES = [1, 5, 15, 30, 60] as const

export type ZoomMinutes = (typeof ZOOM_MINUTES)[number]

export const DEFAULT_ZOOM: ZoomMinutes = 15

export function parseZoom(
  value: string | number | null | undefined,
): ZoomMinutes | undefined {
  return ZOOM_MINUTES.find((zoom) => zoom === Number(value))
}

export function zoomLabel(zoom: ZoomMinutes) {
  return zoom === 60 ? "1h" : `${zoom}m`
}
