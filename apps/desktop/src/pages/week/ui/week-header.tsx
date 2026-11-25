import { ChevronLeftIcon, ChevronRightIcon } from "lucide-react"
import type { ReactNode } from "react"
import { addDays } from "@/shared/lib/local-date"

type WeekHeaderProps = {
  week: string
  title: string
  subtitle: string
  thisWeek: boolean
  today: string
  onWeekChange: (date: string) => void
  totals: ReactNode
}

const arrow =
  "glass grid size-9 place-items-center rounded-full text-t2 outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-40"

export function WeekHeader({
  week,
  title,
  subtitle,
  thisWeek,
  today,
  onWeekChange,
  totals,
}: WeekHeaderProps) {
  return (
    <header className="flex flex-wrap items-end gap-4.5 max-[720px]:gap-2.5">
      <div className="flex flex-col gap-2">
        <span className="flex items-center gap-2">
          <button
            type="button"
            aria-label="Previous week"
            className={arrow}
            onClick={() => onWeekChange(addDays(week, -7))}
          >
            <ChevronLeftIcon aria-hidden="true" className="size-3.5" />
          </button>
          <h1 className="mx-1.5 my-0 text-[42px] leading-none font-light tracking-tight text-t1 max-[720px]:text-[30px]">
            {title}
          </h1>
          <button
            type="button"
            aria-label="Next week"
            className={arrow}
            disabled={thisWeek}
            onClick={() => onWeekChange(addDays(week, 7))}
          >
            <ChevronRightIcon aria-hidden="true" className="size-3.5" />
          </button>
          {!thisWeek && (
            <button
              type="button"
              onClick={() => onWeekChange(today)}
              className="ml-1 h-7.5 rounded-full border border-input px-3 text-t2 outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
            >
              This week
            </button>
          )}
        </span>
        <span className="text-t4">{subtitle}</span>
      </div>
      {totals}
    </header>
  )
}
