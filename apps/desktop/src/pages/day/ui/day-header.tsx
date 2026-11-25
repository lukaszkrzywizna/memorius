import { ChevronLeftIcon, ChevronRightIcon } from "lucide-react"
import type { ReactNode } from "react"
import {
  addDays,
  formatLongDate,
  formatPlainDate,
  parseIsoDate,
} from "@/shared/lib/local-date"

type DayHeaderProps = {
  date: string
  today: string
  onDateChange: (date: string) => void
  /** Shown on the right; wraps below the title on narrow windows. */
  totals: ReactNode
  /** For example the issue the day is filtered by. */
  extra?: ReactNode
}

const arrow =
  "glass grid size-9 place-items-center rounded-full text-t2 outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-40"

export function DayHeader({
  date,
  today,
  onDateChange,
  totals,
  extra,
}: DayHeaderProps) {
  const weekday = parseIsoDate(date).toLocaleDateString("en-GB", {
    weekday: "long",
  })
  return (
    <header className="flex flex-wrap items-start gap-4.5 px-6 pt-5 pb-3 max-[720px]:gap-2.5 max-[720px]:px-0.5 max-[720px]:pt-1.5">
      <div className="flex flex-col gap-2">
        <span className="flex items-center gap-2">
          <button
            type="button"
            aria-label="Previous day"
            className={arrow}
            onClick={() => onDateChange(addDays(date, -1))}
          >
            <ChevronLeftIcon aria-hidden="true" className="size-3.5" />
          </button>
          <h1
            className="mx-1.5 my-0 text-[42px] leading-none font-light tracking-tight text-t1 max-[720px]:text-[30px]"
            aria-label={formatLongDate(date)}
          >
            {weekday}
          </h1>
          <button
            type="button"
            aria-label="Next day"
            className={arrow}
            disabled={date >= today}
            onClick={() => onDateChange(addDays(date, 1))}
          >
            <ChevronRightIcon aria-hidden="true" className="size-3.5" />
          </button>
        </span>
        <span className="flex items-center gap-2.5 text-t4">
          {formatPlainDate(date)}
          {date === today ? " · today" : ""}
          {date !== today && (
            <button
              type="button"
              onClick={() => onDateChange(today)}
              className="text-t3 underline-offset-2 outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring"
            >
              Today
            </button>
          )}
          {extra}
        </span>
      </div>
      {totals}
    </header>
  )
}
