import { AppIcon, windowTitle } from "@/entities/activity"
import type { DayTimeline } from "@/shared/api/generated/DayTimeline"
import type { RecorderStatus } from "@/shared/api/generated/RecorderStatus"
import { topApps } from "@/pages/popover/lib/top-apps"
import { formatDuration } from "@/shared/lib/duration"
import { formatClock } from "@/shared/lib/local-date"
import { Button } from "@/shared/ui/button"
import { Skeleton } from "@/shared/ui/skeleton"

type PopoverPageProps = {
  status: RecorderStatus | undefined
  today: DayTimeline | undefined
  onOpen: () => void
  onPause: () => void
  onResume: () => void
  onQuit: () => void
}

const menuButton = "h-9 justify-start rounded-xl px-3 text-t2"

export function PopoverPage({
  status,
  today,
  onOpen,
  onPause,
  onResume,
  onQuit,
}: PopoverPageProps) {
  const paused = status?.state.kind === "paused"
  const apps = today ? topApps(today.blocks) : []
  const longest = Math.max(1, ...apps.map((total) => total.durationMs))
  const [main = "", ...rest] = today
    ? formatDuration(today.totals.split.workMs).split(" ")
    : []
  return (
    <main className="flex h-full bg-(--backdrop) flex-col gap-2.5 p-3 text-foreground">
      <div className="flex items-center gap-2 px-1 text-t2">
        <span
          aria-hidden="true"
          className={`size-1.5 ${paused ? "bg-t4" : "bg-operational"}`}
        />
        {status?.state.kind === "paused"
          ? `Paused until ${formatClock(status.state.untilMs)}`
          : "Recording"}
      </div>
      <CurrentWindow status={status} />
      <section
        aria-label="Today"
        className="glass flex flex-col gap-2.5 rounded-[18px] px-3.5 py-3"
      >
        <div className="flex items-baseline">
          <h2 className="m-0 text-[13px] font-normal text-t4">
            Your work today
          </h2>
          {!today && <Skeleton className="ml-auto h-7 w-20" />}
          <span className="ml-auto text-[28px] leading-none font-light tracking-tight text-t1">
            {main}
            <span className="text-t5">
              {rest.length ? ` ${rest.join(" ")}` : ""}
            </span>
          </span>
        </div>
        {!today && (
          <div
            role="status"
            aria-label="Loading today"
            className="flex flex-col gap-2"
          >
            {[0, 1, 2, 3].map((row) => (
              <div key={row} className="flex items-center gap-2">
                <Skeleton className="size-3.5 rounded-[4px]" />
                <Skeleton className="h-3 flex-1" />
                <Skeleton className="h-3 w-10" />
              </div>
            ))}
          </div>
        )}
        <ul className="m-0 grid list-none grid-cols-[auto_minmax(0,1fr)_auto_auto] items-center gap-x-2 gap-y-1.5 p-0 text-[13px]">
          {apps.map((total) => (
            <li key={total.app.id} className="contents">
              <AppIcon app={total.app} size={14} />
              <span className="truncate text-t2">{total.app.name}</span>
              <span aria-hidden="true" className="ticks relative h-1.5 w-12">
                <span
                  className="ticks-on absolute inset-y-0 left-0"
                  style={{
                    width: `${Math.round((total.durationMs / longest) * 100)}%`,
                  }}
                />
              </span>
              <span className="text-right text-xs text-t4">
                {formatDuration(total.durationMs)}
              </span>
            </li>
          ))}
        </ul>
      </section>
      <div className="mt-auto flex flex-col">
        <Button variant="ghost" className={menuButton} onClick={onOpen}>
          Open Memorius
        </Button>
        <Button
          variant="ghost"
          className={menuButton}
          onClick={paused ? onResume : onPause}
        >
          {paused ? "Resume recording" : "Pause for 30 minutes"}
        </Button>
        <Button variant="ghost" className={menuButton} onClick={onQuit}>
          Quit Memorius
        </Button>
      </div>
    </main>
  )
}

function CurrentWindow({ status }: { status: RecorderStatus | undefined }) {
  if (status?.state.kind !== "recording" || !status.state.current) {
    return null
  }
  const current = status.state.current
  return (
    <section
      aria-label="Now"
      className="frost flex items-start gap-2.5 rounded-[18px] px-3.5 py-3"
    >
      <AppIcon app={current.app} size={22} />
      <span className="flex min-w-0 flex-col gap-0.5">
        <span className="text-xs text-t4">{current.app.name}</span>
        <span className="break-words text-t1">
          {windowTitle(current.title)}
        </span>
      </span>
    </section>
  )
}
