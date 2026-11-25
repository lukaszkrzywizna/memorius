import { formatDuration } from "@/shared/lib/duration"
import { Skeleton } from "@/shared/ui/skeleton"

export type Stat = {
  label: string
  /** Unknown values show as a dash. */
  ms: number | undefined
  /** A small square before the label, for example `bg-attention`. */
  dot?: string
}

/** Totals as calm glass cards: label, then the time with the minutes dimmed. */
export function StatCards({ label, stats }: { label: string; stats: Stat[] }) {
  return (
    <section
      aria-label={label}
      className="ml-auto flex gap-2 max-[960px]:ml-0 max-[960px]:grid max-[960px]:w-full max-[960px]:grid-cols-[repeat(auto-fit,minmax(120px,1fr))] max-[720px]:grid-cols-2"
    >
      {stats.map((stat) => {
        const [main = "—", ...rest] =
          stat.ms === undefined ? [] : formatDuration(stat.ms).split(" ")
        return (
          <div
            key={stat.label}
            className="glass flex min-w-[108px] flex-col gap-1.5 rounded-[18px] px-3.5 py-2.5 max-[720px]:min-w-0 max-[720px]:px-3 max-[720px]:py-2"
          >
            <span className="flex items-center gap-1.5 text-t4">
              <span
                aria-hidden="true"
                className={`size-1.5 ${stat.dot ?? "bg-transparent"}`}
              />
              {stat.label}
            </span>
            <span className="text-[28px] leading-none font-light tracking-tight text-t1 max-[720px]:text-[22px]">
              {main}
              <span className="text-t5">
                {rest.length ? ` ${rest.join(" ")}` : ""}
              </span>
            </span>
          </div>
        )
      })}
    </section>
  )
}

/** The totals row while its numbers load. */
export function StatCardsSkeleton({ count }: { count: number }) {
  return (
    <div
      aria-hidden="true"
      className="ml-auto flex gap-2 max-[960px]:ml-0 max-[960px]:grid max-[960px]:w-full max-[960px]:grid-cols-[repeat(auto-fit,minmax(120px,1fr))] max-[720px]:grid-cols-2"
    >
      {Array.from({ length: count }, (_, index) => (
        <div
          key={index}
          className="glass flex min-w-[108px] flex-col gap-2.5 rounded-[18px] px-3.5 py-3 max-[720px]:min-w-0"
        >
          <Skeleton className="h-3 w-16" />
          <Skeleton className="h-6 w-20" />
        </div>
      ))}
    </div>
  )
}
