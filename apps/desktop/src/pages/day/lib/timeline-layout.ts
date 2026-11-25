import { formatMinuteOfDay } from "@/shared/lib/local-date"
import type { TimelineView } from "@/pages/day/lib/timeline-view"
import type { ZoomMinutes } from "@/pages/day/lib/zoom"

const HOUR_MS = 3_600_000
/** One slot is this long at every zoom: wider when the time runs sideways. */
export const SLOT_PX: Record<TimelineView, number> = {
  horizontal: 72,
  vertical: 32,
}
/** Room between neighbouring blocks. */
const GAP_PX = 3
const DEFAULT_RANGE = { startHour: 8, endHour: 18 }

/** The visible hours and how many pixels one hour takes. */
export type TimelineScale = {
  view: TimelineView
  startHour: number
  endHour: number
  hourPx: number
  zoom: ZoomMinutes
}
type Span = { startMs: number; endMs: number }

/** 08:00–18:00, widened to whole hours that hold any activity. */
export function timelineScale(
  spans: readonly Span[],
  dayStartMs: number,
  zoom: ZoomMinutes,
  view: TimelineView,
): TimelineScale {
  let { startHour, endHour } = DEFAULT_RANGE
  for (const span of spans) {
    startHour = Math.min(
      startHour,
      Math.floor((span.startMs - dayStartMs) / HOUR_MS),
    )
    endHour = Math.max(endHour, Math.ceil((span.endMs - dayStartMs) / HOUR_MS))
  }
  return {
    view,
    startHour: Math.max(0, startHour),
    endHour: Math.min(24, endHour),
    hourPx: (SLOT_PX[view] * 60) / zoom,
    zoom,
  }
}

/** Where an instant sits along the time axis. */
export function timeOffset(
  ms: number,
  dayStartMs: number,
  scale: TimelineScale,
) {
  return ((ms - dayStartMs) / HOUR_MS - scale.startHour) * scale.hourPx
}

/** A span's place along the time axis, with a gap after it. */
export function spanBox(span: Span, dayStartMs: number, scale: TimelineScale) {
  const size = ((span.endMs - span.startMs) / HOUR_MS) * scale.hourPx
  return {
    start: timeOffset(span.startMs, dayStartMs, scale),
    size: Math.max(4, size - GAP_PX),
  }
}

export type SpanBox = ReturnType<typeof spanBox>

/** Time labels: hourly from 15-minute slots, closer at finer zooms. */
export function timeMarks(scale: TimelineScale) {
  const stepMin = scale.zoom >= 15 ? 60 : scale.zoom >= 5 ? 30 : 15
  const marks = []
  for (
    let minute = scale.startHour * 60;
    minute <= scale.endHour * 60;
    minute += stepMin
  ) {
    marks.push({
      minute,
      label: formatMinuteOfDay(minute),
      offset: (minute / 60 - scale.startHour) * scale.hourPx,
      hour: minute % 60 === 0,
    })
  }
  return marks
}

export function timelineLength(scale: TimelineScale) {
  return (scale.endHour - scale.startHour) * scale.hourPx
}
