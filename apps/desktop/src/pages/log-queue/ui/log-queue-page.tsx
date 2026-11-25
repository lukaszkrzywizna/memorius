import { CheckCheckIcon, CloudUploadIcon } from "lucide-react"
import { useState } from "react"
import type { AgentRun } from "@/shared/api/generated/AgentRun"
import type { AgentSettings } from "@/shared/api/generated/AgentSettings"
import type { DaySummary } from "@/shared/api/generated/DaySummary"
import type { LogResult } from "@/shared/api/generated/LogResult"
import type { ProposalEdit } from "@/shared/api/generated/ProposalEdit"
import type { QueueItem } from "@/shared/api/generated/QueueItem"
import type { RunRange } from "@/shared/api/generated/RunRange"
import {
  type QueueFilter,
  filterCounts,
  queueRows,
} from "@/pages/log-queue/lib/queue-rows"
import { AgentBar } from "@/pages/log-queue/ui/agent-bar"
import { STATE_DOT } from "@/pages/log-queue/ui/state-tone"
import { EmptyQueue } from "@/pages/log-queue/ui/empty-queue"
import { ProposalDetails } from "@/pages/log-queue/ui/proposal-details"
import { QueueSkeleton } from "@/pages/log-queue/ui/queue-skeleton"
import { QueueTable } from "@/pages/log-queue/ui/queue-table"
import { Button } from "@/shared/ui/button"

export type BulkApproval = {
  pending: boolean
  /** Proposals the last bulk approval could not approve, with the reason. */
  failed: { id: number; issue: string; message: string }[]
}

type AgentControls = {
  settings: AgentSettings | undefined
  runs: readonly AgentRun[]
  onPrepare: (range: RunRange) => void
  onStop: () => void
}

/** Logging the approved proposals in YouTrack, done by Memorius itself. */
type YouTrackLogging = {
  /** Logs the given approved proposals (all when none are given), or opens Settings to connect YouTrack. */
  onLog: (ids?: number[]) => void
  connected: boolean
  pending: boolean
  result: LogResult | undefined
  error: string | undefined
}

type LogQueuePageProps = {
  items: readonly QueueItem[] | undefined
  onApprove: (id: number, edit: ProposalEdit | null) => void
  onReject: (id: number) => void
  onUndo: (id: number) => void
  onApproveMany: (ids: number[]) => void
  bulk: BulkApproval
  agent: AgentControls
  youtrack: YouTrackLogging
  /** The tracker's work types, offered when editing a proposal. */
  workTypes: readonly string[]
  /** Totals of the days with open proposals. */
  daySummaries: readonly DaySummary[]
}

const FILTERS: readonly [QueueFilter, string][] = [
  ["all", "All"],
  ["proposed", "To approve"],
  ["approved", "Not logged"],
  ["logged", "Logged"],
  ["rejected", "Rejected"],
]

export function LogQueuePage({
  items,
  onApprove,
  onReject,
  onUndo,
  onApproveMany,
  bulk,
  agent,
  youtrack,
  workTypes,
  daySummaries,
}: LogQueuePageProps) {
  const [filter, setFilter] = useState<QueueFilter>("all")
  const [focusId, setFocusId] = useState<number | null>(null)
  const [checked, setChecked] = useState<ReadonlySet<number>>(new Set())
  const all = items ?? []
  const rows = queueRows(
    all,
    filter,
    new Map(daySummaries.map((summary) => [summary.date, summary])),
  )
  const counts = filterCounts(all)
  const waitingIds = rows.flatMap((row) =>
    row.kind === "item" && row.item.proposal.state.kind === "proposed"
      ? [row.item.proposal.id]
      : [],
  )
  // Only selected rows that are visible and still waiting are approved.
  const chosen = waitingIds.filter((id) => checked.has(id))
  const approvedCount = all.filter(
    (item) => item.proposal.state.kind === "approved",
  ).length
  const focused = all.find((item) => item.proposal.id === focusId)
  const empty = items !== undefined && all.length === 0
  const toggle = (id: number) =>
    setChecked((current) => {
      const next = new Set(current)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  return (
    <div className="relative flex h-full min-h-0 flex-col gap-3 px-1 pt-1.5 pb-2">
      <header className="flex flex-wrap items-end gap-4">
        <div className="flex flex-col gap-1.5">
          <h1 className="m-0 text-[34px] leading-none font-light tracking-tight text-t1">
            Log queue
          </h1>
          <p className="m-0 text-t4">
            Nothing goes to YouTrack before you approve it.
          </p>
        </div>
        {!empty && (
          <>
            <div
              role="group"
              aria-label="Show"
              className="glass ml-auto flex gap-0.5 rounded-full p-[3px] max-[720px]:order-3 max-[720px]:ml-0 max-[720px]:w-full"
            >
              {FILTERS.map(([value, label]) => (
                <button
                  key={value}
                  type="button"
                  aria-pressed={filter === value}
                  onClick={() => setFilter(value)}
                  className="inline-flex h-7.5 items-center justify-center gap-1.5 rounded-full px-3 text-t3 outline-none focus-visible:ring-2 focus-visible:ring-ring aria-pressed:bg-accent aria-pressed:text-foreground max-[720px]:flex-1"
                >
                  {value !== "all" && (
                    <span
                      aria-hidden="true"
                      className={`size-2 rounded-full ${STATE_DOT[value]}`}
                    />
                  )}
                  {label} <span className="text-t4">{counts[value]}</span>
                </button>
              ))}
            </div>
            <Button
              size="lg"
              variant="outline"
              disabled={chosen.length === 0 || bulk.pending}
              onClick={() => {
                onApproveMany(chosen)
                setChecked(new Set())
              }}
            >
              <CheckCheckIcon aria-hidden="true" />
              {bulk.pending
                ? "Approving…"
                : chosen.length > 0
                  ? `Approve ${chosen.length} selected`
                  : "Approve selected"}
            </Button>
            <Button
              size="lg"
              disabled={
                youtrack.connected && (approvedCount === 0 || youtrack.pending)
              }
              onClick={() => youtrack.onLog()}
            >
              <CloudUploadIcon aria-hidden="true" />
              {!youtrack.connected
                ? "Connect YouTrack to log"
                : youtrack.pending
                  ? "Logging…"
                  : approvedCount > 0
                    ? `Log ${approvedCount} to YouTrack`
                    : "Log to YouTrack"}
            </Button>
          </>
        )}
      </header>
      {empty ? (
        <EmptyQueue
          settings={agent.settings}
          latest={agent.runs[0]}
          onPrepare={agent.onPrepare}
          onStop={agent.onStop}
        />
      ) : (
        <AgentBar
          settings={agent.settings}
          latest={agent.runs[0]}
          onPrepare={agent.onPrepare}
          onStop={agent.onStop}
        />
      )}
      <LogOutcome result={youtrack.result} error={youtrack.error} />
      {bulk.failed.length > 0 && (
        <div
          role="alert"
          className="flex flex-wrap items-center gap-3 rounded-2xl border border-critical/50 px-3.5 py-2 text-[13px] text-t1"
        >
          Could not approve{" "}
          {bulk.failed.map((failure) => failure.issue).join(", ")}:{" "}
          {bulk.failed[0]?.message}
          <Button
            variant="outline"
            size="sm"
            className="ml-auto"
            disabled={bulk.pending}
            onClick={() =>
              onApproveMany(bulk.failed.map((failure) => failure.id))
            }
          >
            Retry these
          </Button>
        </div>
      )}
      {!empty && (
        <div
          className={`grid min-h-0 flex-1 items-start gap-3.5 max-[960px]:grid-cols-1 ${
            focused ? "grid-cols-[minmax(0,1fr)_360px]" : "grid-cols-1"
          }`}
        >
          {items === undefined ? (
            <QueueSkeleton />
          ) : rows.length === 0 ? (
            <p className="glass m-0 rounded-[18px] px-5 py-4 text-t4">
              Nothing here with this filter.
            </p>
          ) : (
            <QueueTable
              rows={rows}
              focusId={focusId}
              checked={checked}
              waitingIds={waitingIds}
              onFocus={setFocusId}
              onLog={(id) => youtrack.onLog([id])}
              onToggle={toggle}
              onToggleMany={(ids, on) =>
                setChecked((current) => {
                  const next = new Set(current)
                  for (const id of ids) {
                    if (on) next.add(id)
                    else next.delete(id)
                  }
                  return next
                })
              }
              onApproveMany={onApproveMany}
              onToggleAll={() =>
                setChecked(
                  chosen.length === waitingIds.length
                    ? new Set()
                    : new Set(waitingIds),
                )
              }
              onApprove={(id) => onApprove(id, null)}
              onReject={onReject}
              onUndo={onUndo}
            />
          )}
          {focused && (
            <aside
              aria-label="Proposal details"
              className="glass max-h-full overflow-y-auto rounded-[20px] p-4 max-[960px]:frost max-[960px]:absolute max-[960px]:top-24 max-[960px]:right-0 max-[960px]:z-20 max-[960px]:w-[360px] max-[720px]:fixed max-[720px]:inset-x-2.5 max-[720px]:top-auto max-[720px]:bottom-20 max-[720px]:max-h-[70vh] max-[720px]:w-auto"
            >
              <ProposalDetails
                key={focused.proposal.id}
                item={focused}
                workTypes={workTypes}
                onApprove={onApprove}
                onReject={onReject}
                onUndo={onUndo}
                onClose={() => setFocusId(null)}
              />
            </aside>
          )}
        </div>
      )}
    </div>
  )
}

/** What the last logging by Memorius did: logged, or why some failed. */
function LogOutcome({
  result,
  error,
}: {
  result: LogResult | undefined
  error: string | undefined
}) {
  if (!result && !error) return null
  const failed = result?.failed ?? []
  return (
    <p
      role="status"
      className={`m-0 rounded-2xl border px-3.5 py-2 text-[13px] text-t1 ${
        error || failed.length > 0 ? "border-critical/50" : "border-border"
      }`}
    >
      {error ??
        [
          `Logged ${result?.logged ?? 0} in YouTrack.`,
          ...failed.map(
            (failure) => `${failure.issue} ${failure.date}: ${failure.error}`,
          ),
        ].join(" ")}
    </p>
  )
}
