import { XIcon } from "lucide-react"
import type { AppRef } from "@/shared/api/generated/AppRef"
import { AppIcon } from "@/entities/activity"
import { formatDuration } from "@/shared/lib/duration"
import { Button } from "@/shared/ui/button"

export type DetailRow = {
  label: string
  app?: AppRef
  text: string
  value?: string
  /** 0–100: a tick bar before the value. */
  share?: number
}

type SessionRow = {
  key: string
  app: AppRef
  title: string
  /** How much of the session's agent time counted, or why not. */
  note: string
  private: boolean
  onPrivateChange: (hidden: boolean) => void
}

/** A whole session is either counted or private; minutes are never asked. */
type Privacy = {
  private: boolean
  onPrivateChange: (hidden: boolean) => void
}

export type DetailsCardProps = {
  /** The dialog's name, for example "Selected activity". */
  label: string
  app?: AppRef
  title: string
  sub: string
  chips: { text: string }[]
  durationMs: number
  rows: DetailRow[]
  /** Each agent session in the stretch, each can be marked private. */
  sessions?: SessionRow[]
  privacy?: Privacy
  onClose: () => void
}

/** The selected item's details, next to it on the timeline. */
export function DetailsCard({
  label,
  app,
  title,
  sub,
  chips,
  durationMs,
  rows,
  sessions,
  privacy,
  onClose,
}: DetailsCardProps) {
  const [main = "", ...rest] = formatDuration(durationMs).split(" ")
  return (
    <section
      role="dialog"
      aria-label={label}
      className="frost flex flex-col gap-2.5 rounded-[20px] px-4 py-3.5"
    >
      <div className="flex items-start gap-2.5">
        {app && <AppIcon app={app} size={28} />}
        <span className="flex min-w-0 flex-1 flex-col gap-1">
          <span className="truncate text-[15px] text-t1" title={title}>
            {title}
          </span>
          <span className="flex flex-wrap items-center gap-2 text-xs text-t4">
            {sub}
            {chips.map((chip) => (
              <span
                key={chip.text}
                className="inline-flex h-5 items-center rounded-full border border-input px-2 whitespace-nowrap text-t3"
              >
                {chip.text}
              </span>
            ))}
          </span>
        </span>
        <span className="text-[28px] leading-none font-light tracking-tight whitespace-nowrap text-t1">
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
      {rows.length > 0 && (
        <div className="grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-x-3 gap-y-2 border-t border-border pt-2.5 text-[13px]">
          {rows.map((row, index) => (
            <DetailLine key={`${row.label}:${row.text}:${index}`} row={row} />
          ))}
        </div>
      )}
      {sessions && sessions.length > 0 && (
        <ul
          aria-label="Agent sessions here"
          className="m-0 flex list-none flex-col gap-2 border-t border-border p-0 pt-2.5"
        >
          {sessions.map((session) => (
            <SessionLine key={session.key} session={session} />
          ))}
        </ul>
      )}
      {privacy && (
        <div className="flex items-center gap-3 border-t border-border pt-2.5">
          <p className="m-0 flex-1 text-xs text-t3">
            {privacy.private
              ? "Private: its prompts and agent time do not count."
              : "Not your work? A private session's prompts and agent time do not count."}
          </p>
          <PrivacyButton {...privacy} />
        </div>
      )}
    </section>
  )
}

function DetailLine({ row }: { row: DetailRow }) {
  return (
    <>
      <span className="whitespace-nowrap text-t4">{row.label}</span>
      <span className="flex min-w-0 items-center gap-1.5">
        {row.app && <AppIcon app={row.app} size={14} />}
        <span className="truncate text-t2" title={row.text}>
          {row.text}
        </span>
      </span>
      <span className="flex items-center justify-end gap-2 whitespace-nowrap text-t3">
        {row.share !== undefined && (
          <span aria-hidden="true" className="ticks relative h-1.5 w-16">
            <span
              className="ticks-on absolute inset-y-0 left-0"
              style={{ width: `${row.share}%` }}
            />
          </span>
        )}
        {row.value}
      </span>
    </>
  )
}

function SessionLine({ session }: { session: SessionRow }) {
  return (
    <li className="flex items-center gap-2 text-[13px]">
      <AppIcon app={session.app} size={16} />
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="truncate text-t1" title={session.title}>
          {session.title}
        </span>
        <span className="text-xs text-t4">{session.note}</span>
      </span>
      <PrivacyButton {...session} />
    </li>
  )
}

function PrivacyButton({ private: hidden, onPrivateChange }: Privacy) {
  return (
    <Button
      size="sm"
      variant={hidden ? "ghost" : "outline"}
      onClick={() => onPrivateChange(!hidden)}
    >
      {hidden ? "Count again" : "Mark private"}
    </Button>
  )
}
