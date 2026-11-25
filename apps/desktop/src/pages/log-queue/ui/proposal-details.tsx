import { XIcon } from "lucide-react"
import { useState } from "react"
import type { ProposalEdit } from "@/shared/api/generated/ProposalEdit"
import type { QueueItem } from "@/shared/api/generated/QueueItem"
import { AppIcon } from "@/entities/activity"
import { proposerApp } from "@/pages/log-queue/lib/queue-rows"
import { ProposalEditForm } from "@/pages/log-queue/ui/proposal-edit-form"
import { formatDuration } from "@/shared/lib/duration"
import {
  formatClock,
  formatShortDate,
  parseIsoDate,
} from "@/shared/lib/local-date"
import { Button } from "@/shared/ui/button"

type ProposalDetailsProps = {
  item: QueueItem
  workTypes: readonly string[]
  onApprove: (id: number, edit: ProposalEdit | null) => void
  onReject: (id: number) => void
  onUndo: (id: number) => void
  onClose: () => void
}

/** One proposal in full: text, evidence, and the decision with an inline edit. */
export function ProposalDetails({
  item,
  workTypes,
  onApprove,
  onReject,
  onUndo,
  onClose,
}: ProposalDetailsProps) {
  const { proposal } = item
  const [editing, setEditing] = useState(false)
  const [main = "", ...rest] = formatDuration(proposal.minutes * 60_000).split(
    " ",
  )
  const longest = Math.max(1, ...item.apps.map((usage) => usage.durationMs))
  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-start gap-2.5">
        <span className="flex flex-1 flex-col gap-1">
          <span className="text-[15px] text-t1">{proposal.issue}</span>
          <span className="text-xs text-t4">
            {formatShortDate(parseIsoDate(proposal.date).getTime())} · work{" "}
            {formatClock(proposal.startMs)}–{formatClock(proposal.endMs)}
            {proposal.workType ? ` · ${proposal.workType}` : ""}
          </span>
        </span>
        <span className="text-[28px] leading-none font-light tracking-tight text-t1">
          {main}
          <span className="text-t5">
            {rest.length ? ` ${rest.join(" ")}` : ""}
          </span>
        </span>
        <button
          type="button"
          aria-label="Close details"
          onClick={onClose}
          className="-mt-1 -mr-1.5 grid size-7 place-items-center rounded-full text-t3 outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
        >
          <XIcon aria-hidden="true" className="size-3" />
        </button>
      </div>
      <span className="flex items-center gap-1.5 text-xs text-t3">
        <AppIcon app={proposerApp(proposal.agent)} size={14} />
        Proposed by {proposal.agent}
      </span>
      {editing ? (
        <ProposalEditForm
          proposal={proposal}
          workTypes={workTypes}
          onApprove={(edit) => {
            onApprove(proposal.id, edit)
            setEditing(false)
          }}
          onCancel={() => setEditing(false)}
        />
      ) : (
        <p className="m-0 leading-normal whitespace-pre-line text-t2">
          {proposal.text}
        </p>
      )}
      <ul
        aria-label="Recorded apps"
        className="m-0 grid list-none grid-cols-[auto_minmax(0,1fr)_auto_auto] items-center gap-x-2.5 gap-y-2 border-t border-border p-0 pt-2.5 text-[13px]"
      >
        {item.apps.map((usage) => (
          <li key={usage.app.id} className="contents">
            <AppIcon app={usage.app} size={16} />
            <span className="truncate text-t2">{usage.app.name}</span>
            <span aria-hidden="true" className="ticks relative h-1.5 w-16">
              <span
                className="ticks-on absolute inset-y-0 left-0"
                style={{
                  width: `${Math.round((usage.durationMs / longest) * 100)}%`,
                }}
              />
            </span>
            <span className="text-right text-t3">
              {formatDuration(usage.durationMs)}
            </span>
          </li>
        ))}
        {item.apps.length === 0 && (
          <li className="col-span-4 text-xs text-destructive">
            No activity was recorded in this time.
          </li>
        )}
      </ul>
      {proposal.state.kind === "approved" && proposal.lastError && (
        <p role="alert" className="m-0 text-xs text-destructive">
          Last attempt to log failed: {proposal.lastError}
        </p>
      )}
      {proposal.state.kind === "proposed" && !editing && (
        <div className="flex gap-2">
          <Button
            className="flex-1"
            onClick={() => onApprove(proposal.id, null)}
          >
            Approve
          </Button>
          <Button variant="outline" onClick={() => setEditing(true)}>
            Edit
          </Button>
          <Button variant="ghost" onClick={() => onReject(proposal.id)}>
            Reject
          </Button>
        </div>
      )}
      {(proposal.state.kind === "approved" ||
        proposal.state.kind === "rejected") && (
        <Button
          variant="outline"
          size="sm"
          className="self-start"
          onClick={() => onUndo(proposal.id)}
        >
          Undo
        </Button>
      )}
    </div>
  )
}
