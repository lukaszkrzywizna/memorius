import { ChevronRightIcon, ListIcon } from "lucide-react"
import type { NeedsYou } from "@/app/shell/model/needs-you"
import { formatDuration } from "@/shared/lib/duration"

type NeedsYouPanelProps = {
  needs: NeedsYou
  onOpenQueue: () => void
}

/** Today's open questions: proposals to approve and issues not logged yet. */
export function NeedsYouPanel({ needs, onOpenQueue }: NeedsYouPanelProps) {
  const empty = needs.waiting.length === 0 && needs.unlogged.length === 0
  return (
    <div className="flex flex-col gap-4">
      {empty && (
        <p className="m-0 text-t3">
          Nothing needs you now. New proposals and time not logged yet show
          here.
        </p>
      )}
      {needs.waiting.length > 0 && (
        <section aria-label="Proposals" className="flex flex-col gap-2">
          <SectionTitle icon={<ListIcon />}>
            Proposals{" "}
            <span className="text-t4">({needs.waiting.length} waiting)</span>
          </SectionTitle>
          {needs.waiting.slice(0, 3).map((proposal) => (
            <button
              key={proposal.id}
              type="button"
              onClick={onOpenQueue}
              className="flex items-center gap-2.5 rounded-2xl border border-(--glass-edge) bg-warm-item px-3 py-2 text-left outline-none hover:bg-warm focus-visible:ring-2 focus-visible:ring-ring"
            >
              <span className="flex min-w-0 flex-1 flex-col">
                <span>
                  {proposal.issue} · {formatDuration(proposal.minutes * 60_000)}
                </span>
                <span className="truncate text-xs text-t4">
                  {proposal.text}
                </span>
              </span>
              <ChevronRightIcon
                aria-hidden="true"
                className="size-3.5 text-t4"
              />
            </button>
          ))}
        </section>
      )}
      {needs.unlogged.length > 0 && (
        <section
          aria-label="Not logged yet"
          className="flex flex-col gap-2 border-t border-border pt-3"
        >
          <span className="flex items-baseline justify-between">
            <span className="text-t3">Not logged yet</span>
            <span className="text-[22px] font-light">
              {formatDuration(needs.unloggedMs)}
            </span>
          </span>
          <ul className="m-0 flex list-none flex-col gap-1.5 p-0">
            {needs.unlogged.map((row) => (
              <li
                key={row.issue}
                className="grid grid-cols-[64px_minmax(0,1fr)_56px] items-center gap-2"
              >
                <span>{row.issue}</span>
                <span className="ticks h-2">
                  <span
                    className="ticks-attention block h-full"
                    style={{
                      width: `${Math.round((row.minutes / needs.unlogged[0]!.minutes) * 100)}%`,
                    }}
                  />
                </span>
                <span className="text-right">
                  {formatDuration(row.minutes * 60_000)}
                </span>
              </li>
            ))}
          </ul>
        </section>
      )}
    </div>
  )
}

function SectionTitle({
  icon,
  children,
}: {
  icon: React.ReactNode
  children: React.ReactNode
}) {
  return (
    <h3 className="m-0 flex items-center gap-2 text-[13px] font-normal text-t3 [&_svg]:size-3.5">
      {icon}
      <span>{children}</span>
    </h3>
  )
}
