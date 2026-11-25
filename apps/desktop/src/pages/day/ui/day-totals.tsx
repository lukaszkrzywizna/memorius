import type { Totals } from "@/shared/api/generated/Totals"
import { StatCards } from "@/shared/ui/stat-cards"

type DayTotalsProps = {
  totals: Totals
  /** Active time with no proposal; unknown until the week data loads. */
  unloggedMs: number | undefined
}

/** Your work, not logged, and agent time counted or not, as in the Vexto cards. */
export function DayTotals({ totals, unloggedMs }: DayTotalsProps) {
  return (
    <StatCards
      label="Day totals"
      stats={[
        { label: "Your work", ms: totals.split.workMs, dot: "bg-operational" },
        { label: "Not logged", ms: unloggedMs },
        { label: "Agents counted", ms: totals.split.agentMs },
        { label: "Not counted", ms: totals.split.uncountedMs },
      ]}
    />
  )
}
