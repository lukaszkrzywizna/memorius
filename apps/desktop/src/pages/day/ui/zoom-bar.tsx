import { CrosshairIcon } from "lucide-react"
import type { TimelineView } from "@/pages/day/lib/timeline-view"
import { ZOOM_MINUTES, type ZoomMinutes, zoomLabel } from "@/pages/day/lib/zoom"

type ZoomBarProps = {
  zoom: ZoomMinutes
  onZoomChange: (zoom: ZoomMinutes) => void
  view: TimelineView
  onViewChange: (view: TimelineView) => void
  /** Scrolls to the current time; missing on other days. */
  onNow?: () => void
}

const VIEWS: readonly [TimelineView, string, string][] = [
  ["horizontal", "Horizontal timeline", "M4 7h9M8 12h12M4 17h7"],
  ["vertical", "Vertical timeline", "M7 4v9M12 8v12M17 4v7"],
]

const option =
  "h-9 rounded-full border-0 outline-none focus-visible:ring-2 focus-visible:ring-ring aria-pressed:bg-accent aria-pressed:text-foreground"

/** Zoom, timeline direction and "now", floating over the timeline. */
export function ZoomBar({
  zoom,
  onZoomChange,
  view,
  onViewChange,
  onNow,
}: ZoomBarProps) {
  return (
    <div
      role="toolbar"
      aria-label="Timeline view"
      className="glass absolute right-[18px] bottom-4 z-20 flex items-center gap-1 overflow-x-auto rounded-full p-1 text-t4 max-[720px]:inset-x-0 max-[720px]:bottom-1"
    >
      {ZOOM_MINUTES.map((minutes) => (
        <button
          key={minutes}
          type="button"
          aria-pressed={zoom === minutes}
          aria-label={`Zoom ${zoomLabel(minutes)} slots`}
          onClick={() => onZoomChange(minutes)}
          className={`${option} min-w-10 px-2 max-[720px]:min-w-0`}
        >
          {zoomLabel(minutes)}
        </button>
      ))}
      <Divider />
      {VIEWS.map(([value, label, path]) => (
        <button
          key={value}
          type="button"
          aria-pressed={view === value}
          aria-label={label}
          title={label}
          onClick={() => onViewChange(value)}
          className={`${option} grid w-10 place-items-center`}
        >
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.6"
            strokeLinecap="round"
            aria-hidden="true"
          >
            <path d={path} />
          </svg>
        </button>
      ))}
      {onNow && (
        <>
          <Divider />
          <button
            type="button"
            onClick={onNow}
            className="inline-flex h-9 items-center gap-1.5 rounded-full border border-input px-3 text-t2 outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
          >
            <CrosshairIcon aria-hidden="true" className="size-3.5" />
            now
          </button>
        </>
      )}
    </div>
  )
}

function Divider() {
  return (
    <span
      aria-hidden="true"
      className="mx-0.5 my-1.5 w-px self-stretch bg-t1/10"
    />
  )
}
