import type { WeekCell } from "@/shared/api/generated/WeekCell"
import type { WeekSummary } from "@/shared/api/generated/WeekSummary"
import { formatDuration } from "@/shared/lib/duration"
import { shortDay, visibleDays } from "@/pages/week/lib/week-days"

const cellStyle: Record<WeekCell["kind"], string> = {
  logged: "border-transparent bg-accent text-t1",
  proposed: "border-attention/70 bg-attention/8 text-t1",
  detected: "border-dashed border-t5 text-t3",
  empty: "border-transparent text-t5",
}

const legend = [
  { label: "Logged in YouTrack", swatch: "bg-accent" },
  { label: "Proposed or approved", swatch: "border border-attention/70" },
  { label: "Detected, no proposal", swatch: "border border-dashed border-t5" },
]

function cellMinutes(cell: WeekCell) {
  switch (cell.kind) {
    case "empty":
      return 0
    case "logged":
    case "proposed":
    case "detected":
      return cell.minutes
  }
}

type TimesheetProps = {
  summary: WeekSummary
  /** An issue to mark, for example from search. */
  highlight: string | null
}

/** Issues by day, with how each cell stands in YouTrack. */
export function Timesheet({ summary, highlight }: TimesheetProps) {
  const days = visibleDays(summary.days)
  const head = "px-2 py-3 text-left font-normal text-t4"
  return (
    <section
      aria-label="Timesheet"
      className="glass rounded-[22px] px-2 pt-1.5 pb-2.5"
    >
      <div className="overflow-x-auto">
        <table className="w-full min-w-[620px] border-collapse text-[13px]">
          <thead>
            <tr>
              <th scope="col" className={`${head} px-3.5`}>
                Issue
              </th>
              {days.map(({ day }) => (
                <th key={day.date} scope="col" className={head}>
                  {shortDay(day.date)}
                </th>
              ))}
              <th scope="col" className={`${head} px-3.5 text-right`}>
                Total
              </th>
            </tr>
          </thead>
          <tbody>
            {summary.issues.map((issue) => (
              <tr
                key={issue.issue}
                className={`border-t border-border ${issue.issue === highlight ? "bg-accent/70" : ""}`}
              >
                <th
                  scope="row"
                  className="sticky left-0 bg-transparent px-3.5 py-2.5 text-left font-medium text-t1 max-[720px]:bg-card"
                >
                  {issue.issue}
                </th>
                {days.map(({ day, index }) => {
                  const cell = issue.cells[index] ?? { kind: "empty" as const }
                  return (
                    <td key={day.date} className="p-2">
                      <span
                        className={`inline-flex h-6.5 min-w-16 items-center justify-center rounded-full border px-2.5 ${cellStyle[cell.kind]}`}
                      >
                        {cell.kind === "empty"
                          ? "—"
                          : formatDuration(cellMinutes(cell) * 60_000)}
                      </span>
                    </td>
                  )
                })}
                <td className="px-3.5 py-2.5 text-right text-t2">
                  {formatDuration(
                    issue.cells.reduce(
                      (sum, cell) => sum + cellMinutes(cell),
                      0,
                    ) * 60_000,
                  )}
                </td>
              </tr>
            ))}
            <tr className="border-t border-t1/10">
              <th
                scope="row"
                className="px-3.5 py-3 text-left font-normal text-t4"
              >
                Tracked
              </th>
              {days.map(({ day }) => (
                <td key={day.date} className="px-2 py-3 text-t3">
                  {formatDuration(day.activeMs)}
                </td>
              ))}
              <td className="px-3.5 py-3 text-right text-t3">
                {formatDuration(
                  summary.days.reduce((sum, day) => sum + day.activeMs, 0),
                )}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <ul className="m-0 flex list-none flex-wrap gap-x-4.5 gap-y-2 px-3.5 pt-2 pb-1 text-xs text-t4">
        {legend.map((item) => (
          <li key={item.label} className="flex items-center gap-1.5">
            <span
              aria-hidden="true"
              className={`h-2.5 w-3.5 rounded-full ${item.swatch}`}
            />
            {item.label}
          </li>
        ))}
      </ul>
    </section>
  )
}
