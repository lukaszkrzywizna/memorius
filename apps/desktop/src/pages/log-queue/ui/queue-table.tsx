import {
  CheckCheckIcon,
  CheckIcon,
  CloudUploadIcon,
  ChevronDownIcon,
  Undo2Icon,
  XIcon,
} from "lucide-react"
import { useState } from "react"
import type { QueueItem } from "@/shared/api/generated/QueueItem"
import { AppIcon } from "@/entities/activity"
import { STATE_DOT } from "@/pages/log-queue/ui/state-tone"
import { type QueueRow, proposerApp } from "@/pages/log-queue/lib/queue-rows"
import { formatDuration } from "@/shared/lib/duration"
import { formatShortDate, parseIsoDate } from "@/shared/lib/local-date"

const COLUMNS =
  "grid grid-cols-[28px_84px_132px_26px_minmax(0,1fr)_120px_64px_150px] items-center gap-x-2.5 max-[960px]:grid-cols-[24px_64px_minmax(0,1fr)_auto] max-[960px]:[grid-template-areas:'sel_key_when_dur'_'sel_desc_desc_desc'_'sel_ev_act_act'] max-[960px]:gap-y-1.5"

type QueueTableProps = {
  rows: readonly QueueRow[]
  focusId: number | null
  checked: ReadonlySet<number>
  /** Waiting ids shown now; the header box selects these only. */
  waitingIds: readonly number[]
  onFocus: (id: number) => void
  onToggle: (id: number) => void
  /** Selects (or clears) these waiting proposals together. */
  onToggleMany: (ids: readonly number[], on: boolean) => void
  onToggleAll: () => void
  onApproveMany: (ids: number[]) => void
  onApprove: (id: number) => void
  onReject: (id: number) => void
  onUndo: (id: number) => void
  /** Logs one approved proposal in YouTrack. */
  onLog: (id: number) => void
}

/** Every proposal on one line or two, grouped by who acts next. */
export function QueueTable({
  rows,
  focusId,
  checked,
  waitingIds,
  onFocus,
  onToggle,
  onToggleMany,
  onToggleAll,
  onApproveMany,
  onApprove,
  onReject,
  onUndo,
  onLog,
}: QueueTableProps) {
  const [collapsed, setCollapsed] = useState<ReadonlySet<string>>(new Set())
  const allChecked =
    waitingIds.length > 0 && waitingIds.every((id) => checked.has(id))
  return (
    <div
      role="table"
      aria-label="Proposals"
      className="glass min-h-0 overflow-y-auto rounded-[18px] py-1 [scrollbar-width:thin]"
    >
      <div
        role="row"
        className={`${COLUMNS} h-8.5 border-b border-border px-3.5 text-xs text-t4 max-[960px]:hidden`}
      >
        <span role="columnheader">
          <input
            type="checkbox"
            aria-label="Select all waiting"
            checked={allChecked}
            disabled={waitingIds.length === 0}
            onChange={onToggleAll}
            className="size-3.5 accent-(--t1)"
          />
        </span>
        <span role="columnheader">Issue</span>
        <span role="columnheader">Day</span>
        <span role="columnheader" aria-label="Agent" />
        <span role="columnheader">Description</span>
        <span role="columnheader">Evidence</span>
        <span role="columnheader" className="text-right">
          Time
        </span>
        <span role="columnheader" aria-label="Actions" />
      </div>
      {rows.map((row) => {
        switch (row.kind) {
          case "group":
            return (
              <div
                key={row.title}
                role="row"
                className="flex h-8 items-center gap-2 border-b border-border bg-muted px-3.5 text-xs font-medium text-t2"
              >
                <span
                  aria-hidden="true"
                  className={`size-2 rounded-full ${STATE_DOT[row.state]}`}
                />
                <span role="cell">{row.title}</span>
                <span className="font-normal text-t4">· {row.count}</span>
                <span role="cell" className="ml-auto text-t1">
                  {formatDuration(row.minutes * 60_000)}
                </span>
              </div>
            )
          case "day": {
            const waiting = row.ids.filter((id) => waitingIds.includes(id))
            return (
              <DayRow
                key={row.group}
                row={row}
                open={!collapsed.has(row.group)}
                waiting={waiting}
                checked={
                  waiting.length > 0 && waiting.every((id) => checked.has(id))
                }
                onToggleOpen={() =>
                  setCollapsed((current) => {
                    const next = new Set(current)
                    if (next.has(row.group)) next.delete(row.group)
                    else next.add(row.group)
                    return next
                  })
                }
                onToggleMany={onToggleMany}
                onApproveMany={onApproveMany}
              />
            )
          }
          case "item":
            if (collapsed.has(row.day)) return null
            return (
              <ItemRow
                key={row.item.proposal.id}
                item={row.item}
                focused={focusId === row.item.proposal.id}
                checked={checked.has(row.item.proposal.id)}
                onFocus={onFocus}
                onToggle={onToggle}
                onApprove={onApprove}
                onReject={onReject}
                onUndo={onUndo}
                onLog={onLog}
              />
            )
        }
      })}
    </div>
  )
}

type DayRowProps = {
  row: Extract<QueueRow, { kind: "day" }>
  open: boolean
  /** The day's proposals still waiting for the user. */
  waiting: readonly number[]
  checked: boolean
  onToggleOpen: () => void
} & Pick<QueueTableProps, "onToggleMany" | "onApproveMany">

/**
 * A day's group row in the table's own columns: its sum to log under Time,
 * its work, YouTrack time and time not covered under Description; the
 * proposals of the day sit under it.
 */
function DayRow({
  row,
  open,
  waiting,
  checked,
  onToggleOpen,
  onToggleMany,
  onApproveMany,
}: DayRowProps) {
  const { summary } = row
  const day = formatShortDate(parseIsoDate(row.date).getTime())
  const count = `${row.ids.length} worklog${row.ids.length === 1 ? "" : "s"}`
  return (
    <div
      role="row"
      aria-label={`${day}, ${formatDuration(row.minutes * 60_000)} to log`}
      className={`${COLUMNS} min-h-10 border-b border-border bg-muted/60 px-3.5 text-[13px] max-[960px]:flex max-[960px]:flex-wrap max-[960px]:py-2`}
    >
      <span role="cell">
        {waiting.length > 0 && (
          <input
            type="checkbox"
            aria-label={`Select every waiting worklog of ${day}`}
            checked={checked}
            onChange={() => onToggleMany(waiting, !checked)}
            className="size-3.5 accent-(--t1)"
          />
        )}
      </span>
      <span role="cell" className="col-span-2">
        <button
          type="button"
          aria-expanded={open}
          onClick={onToggleOpen}
          className="inline-flex items-center gap-1.5 rounded-md text-t1 outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <ChevronDownIcon
            aria-hidden="true"
            className={`size-3.5 text-t4 transition-transform ${open ? "" : "-rotate-90"}`}
          />
          {day}
        </button>
      </span>
      <span role="cell" className="max-[960px]:hidden" />
      <span role="cell" className="truncate text-xs text-t3">
        {count}
        {summary &&
          ` · worked ${formatDuration(summary.workMs)} · in YouTrack ${formatDuration(summary.loggedMs)} · not covered ${formatDuration(summary.unloggedMs)}`}
      </span>
      <span role="cell" className="max-[960px]:hidden" />
      <span
        role="cell"
        className="text-right text-[15px] font-medium whitespace-nowrap text-t1 max-[960px]:ml-auto"
      >
        {formatDuration(row.minutes * 60_000)}
      </span>
      <span role="cell" className="flex justify-end">
        {waiting.length > 0 && (
          <button
            type="button"
            onClick={() => onApproveMany([...waiting])}
            className="inline-flex h-6.5 items-center gap-1 rounded-full border border-input px-2.5 text-xs text-t2 outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
          >
            <CheckCheckIcon aria-hidden="true" className="size-3.5" />
            Approve day
          </button>
        )}
      </span>
    </div>
  )
}

/** A row's state: the bar on its left edge, its tint, and fading. */
const ROW_TONE = {
  proposed: { bar: "border-l-pending", tint: "", fade: "" },
  approved: { bar: "border-l-attention", tint: "bg-attention/8", fade: "" },
  failed: { bar: "border-l-critical", tint: "bg-critical/8", fade: "" },
  logged: { bar: "border-l-operational", tint: "", fade: "opacity-60" },
  rejected: { bar: "border-l-t5", tint: "", fade: "opacity-55" },
} as const

type ItemRowProps = Pick<
  QueueTableProps,
  "onFocus" | "onToggle" | "onApprove" | "onReject" | "onUndo" | "onLog"
> & {
  item: QueueItem
  focused: boolean
  checked: boolean
}

function ItemRow({
  item,
  focused,
  checked,
  onFocus,
  onToggle,
  onApprove,
  onReject,
  onUndo,
  onLog,
}: ItemRowProps) {
  const { proposal } = item
  // Trackers log per issue and day; the hours stay in the details.
  const when = formatShortDate(parseIsoDate(proposal.date).getTime())
  const state = proposal.state
  const failed = state.kind === "approved" && proposal.lastError !== null
  const tone = ROW_TONE[failed ? "failed" : state.kind]
  const above = "relative z-10"
  return (
    <div
      role="row"
      aria-selected={focused}
      className={`${COLUMNS} relative min-h-11 border-b border-l-3 border-border px-3.5 pl-[11px] text-[13px] max-[960px]:py-2.5 ${
        tone.bar
      } ${focused ? "bg-accent" : `${tone.tint} hover:bg-muted`} ${tone.fade}`}
    >
      <button
        type="button"
        aria-label={`Details of ${proposal.issue}, ${when}`}
        onClick={() => onFocus(proposal.id)}
        className="absolute inset-0 z-0 outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset"
      />
      <span
        role="cell"
        className={`${above} max-[960px]:[grid-area:sel] max-[960px]:self-start`}
      >
        {state.kind === "proposed" && (
          <input
            type="checkbox"
            aria-label={`Select ${proposal.issue}, ${when}`}
            checked={checked}
            onChange={() => onToggle(proposal.id)}
            className="size-3.5 accent-(--t1)"
          />
        )}
      </span>
      <span
        role="cell"
        className="pointer-events-none pl-5 text-t1 max-[960px]:[grid-area:key]"
      >
        {proposal.issue}
      </span>
      <span role="cell" />
      <span role="cell" className="pointer-events-none max-[960px]:hidden">
        <AppIcon app={proposerApp(proposal.agent)} size={16} />
      </span>
      <span
        role="cell"
        className={`pointer-events-none truncate text-t2 max-[960px]:line-clamp-2 max-[960px]:whitespace-normal max-[960px]:[grid-area:desc] ${
          state.kind === "rejected" ? "line-through" : ""
        }`}
      >
        {proposal.text}
      </span>
      <span
        role="cell"
        className="pointer-events-none flex items-center gap-1.5 overflow-hidden text-xs whitespace-nowrap text-t4 max-[960px]:[grid-area:ev]"
      >
        {item.apps.slice(0, 2).map((usage) => (
          <span key={usage.app.id} className="inline-flex items-center gap-1">
            <AppIcon app={usage.app} size={14} />
            {formatDuration(usage.durationMs)}
          </span>
        ))}
      </span>
      <span
        role="cell"
        className="pointer-events-none text-right text-[15px] whitespace-nowrap text-t1 max-[960px]:[grid-area:dur]"
      >
        {formatDuration(proposal.minutes * 60_000)}
      </span>
      <span
        role="cell"
        className={`${above} flex items-center justify-end gap-1 max-[960px]:[grid-area:act]`}
      >
        <RowActions
          item={item}
          onApprove={onApprove}
          onReject={onReject}
          onUndo={onUndo}
          onLog={onLog}
        />
      </span>
    </div>
  )
}

function RowActions({
  item,
  onApprove,
  onReject,
  onUndo,
  onLog,
}: Pick<ItemRowProps, "item" | "onApprove" | "onReject" | "onUndo" | "onLog">) {
  const { proposal } = item
  const label = `${proposal.issue}, ${formatShortDate(parseIsoDate(proposal.date).getTime())}`
  const undo = (
    <button
      type="button"
      aria-label={`Undo ${label}`}
      onClick={() => onUndo(proposal.id)}
      className="inline-flex h-6.5 items-center gap-1 rounded-full border border-input px-2.5 text-xs text-t2 outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
    >
      <Undo2Icon aria-hidden="true" className="size-3.5" />
      Undo
    </button>
  )
  const status = (pill: string, text: string, withUndo: boolean) => (
    <>
      <span
        className={`inline-flex h-6 items-center rounded-full px-2.5 text-xs whitespace-nowrap ${pill}`}
      >
        {text}
      </span>
      {withUndo && undo}
    </>
  )
  switch (proposal.state.kind) {
    case "proposed":
      return (
        <>
          <button
            type="button"
            aria-label={`Approve ${label}`}
            onClick={() => onApprove(proposal.id)}
            className="inline-flex h-7 items-center gap-1 rounded-full bg-primary px-3 text-xs text-primary-foreground outline-none hover:bg-primary/85 focus-visible:ring-2 focus-visible:ring-ring"
          >
            <CheckIcon aria-hidden="true" className="size-3.5" />
            Approve
          </button>
          <button
            type="button"
            aria-label={`Reject ${label}`}
            title="Reject"
            onClick={() => onReject(proposal.id)}
            className="grid size-7 place-items-center rounded-full border border-input text-t3 outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
          >
            <XIcon aria-hidden="true" className="size-3" />
          </button>
        </>
      )
    case "approved":
      return (
        <>
          {proposal.lastError && (
            <span
              title={proposal.lastError}
              className="inline-flex h-6 items-center rounded-full bg-critical/15 px-2.5 text-xs whitespace-nowrap text-t1"
            >
              Log failed
            </span>
          )}
          <button
            type="button"
            aria-label={`Log ${label} to YouTrack`}
            onClick={() => onLog(proposal.id)}
            className="inline-flex h-7 items-center gap-1 rounded-full bg-primary px-3 text-xs text-primary-foreground outline-none hover:bg-primary/85 focus-visible:ring-2 focus-visible:ring-ring"
          >
            <CloudUploadIcon aria-hidden="true" className="size-3.5" />
            {proposal.lastError ? "Retry" : "Log"}
          </button>
          {undo}
        </>
      )
    case "rejected":
      return status("bg-muted text-t3", "Rejected", true)
    case "logged":
      return status(
        "bg-operational/15 text-t1",
        proposal.state.workItemId
          ? `Logged · ${proposal.state.workItemId}`
          : "Logged",
        false,
      )
  }
}
