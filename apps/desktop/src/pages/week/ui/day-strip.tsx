import { ArrowUpRightIcon } from "lucide-react"
import type { WeekDay } from "@/shared/api/generated/WeekDay"
import { formatDuration } from "@/shared/lib/duration"
import { dayCardLabels, sparkline } from "@/pages/week/lib/week-days"

type DayStripProps = {
  days: readonly WeekDay[]
  today: string
  onOpenDay: (date: string) => void
}

const share = (part: number, whole: number) =>
  whole > 0 ? Math.min(100, Math.round((part / whole) * 100)) : 0

/** One card per day: time, its shape over the day, and what is logged. */
export function DayStrip({ days, today, onOpenDay }: DayStripProps) {
  return (
    <section
      aria-label="Days"
      className="grid grid-cols-[repeat(auto-fit,minmax(150px,1fr))] gap-3 max-[720px]:flex max-[720px]:snap-x max-[720px]:snap-mandatory max-[720px]:overflow-x-auto max-[720px]:pb-1"
    >
      {days.map((day) => {
        const total = Math.max(day.activeMs, day.loggedMs + day.proposedMs)
        const { name, date } = dayCardLabels(day.date)
        const [main = "", ...rest] = formatDuration(day.activeMs).split(" ")
        return (
          <button
            key={day.date}
            type="button"
            aria-label={`${name}, ${formatDuration(day.activeMs)}, open the day`}
            onClick={() => onOpenDay(day.date)}
            className={`glass flex flex-col gap-2.5 rounded-[20px] px-4 py-3.5 text-left outline-none hover:border-ring/40 focus-visible:ring-2 focus-visible:ring-ring max-[720px]:min-w-[150px] max-[720px]:flex-[0_0_44%] max-[720px]:snap-start ${
              day.date === today ? "border-ring/40" : ""
            }`}
          >
            <span className="flex w-full items-start justify-between">
              <span className="flex flex-col gap-0.5">
                <span className="text-[15px] text-t1">{name}</span>
                <span className="text-xs text-t4">{date}</span>
              </span>
              <ArrowUpRightIcon
                aria-hidden="true"
                className="size-3.5 text-t4"
              />
            </span>
            <span className="text-[32px] leading-none font-light tracking-tight text-t1 max-[720px]:text-2xl">
              {main}
              <span className="text-t5">
                {rest.length ? ` ${rest.join(" ")}` : ""}
              </span>
            </span>
            <svg
              viewBox="0 0 100 26"
              preserveAspectRatio="none"
              aria-hidden="true"
              className="h-6.5 w-full"
            >
              <polyline
                points={sparkline(day.hourlyActiveMs)}
                fill="none"
                className="stroke-t2"
                strokeWidth="1.2"
                vectorEffect="non-scaling-stroke"
              />
            </svg>
            <span
              aria-hidden="true"
              className="ticks relative block h-2 w-full"
            >
              <span
                className="ticks-on absolute inset-y-0 left-0"
                style={{ width: `${share(day.loggedMs, total)}%` }}
              />
              <span
                className="ticks-attention absolute inset-y-0"
                style={{
                  left: `${share(day.loggedMs, total)}%`,
                  width: `${share(day.proposedMs, total)}%`,
                }}
              />
            </span>
            <span className="flex w-full justify-between text-xs text-t4">
              <span>{formatDuration(day.loggedMs)} logged</span>
              <span>{formatDuration(day.unloggedMs)} open</span>
            </span>
          </button>
        )
      })}
    </section>
  )
}
