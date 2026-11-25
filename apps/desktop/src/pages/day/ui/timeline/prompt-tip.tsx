import type { ComponentType } from "react"
import type { SessionRef } from "@/shared/api/generated/SessionRef"
import { AppIcon } from "@/entities/activity"
import type { PromptMark } from "@/pages/day/ui/timeline/timeline-model"
import { formatClock } from "@/shared/lib/local-date"

/** Shows one prompt's text; the app reads it from the session log on demand. */
export type PromptTextView = ComponentType<{
  sessions: SessionRef[]
  atMs: number
}>

type PromptTipProps = {
  prompt: PromptMark
  PromptText: PromptTextView
  style: React.CSSProperties
}

/** The hover card of a prompt tick: session, time and what the user wrote. */
export function PromptTip({ prompt, PromptText, style }: PromptTipProps) {
  return (
    <div
      role="tooltip"
      className="frost pointer-events-none absolute z-30 flex w-80 flex-col gap-2 rounded-2xl px-3.5 py-3"
      style={style}
    >
      <span className="flex items-center gap-2 text-xs text-t4">
        <AppIcon app={prompt.app} size={14} />
        <span className="min-w-0 truncate text-t2">{prompt.session}</span>
        <span className="ml-auto shrink-0">{formatClock(prompt.atMs)}</span>
      </span>
      <span className="line-clamp-6 text-[13px] leading-normal whitespace-pre-line text-t1">
        <PromptText sessions={prompt.sessions} atMs={prompt.atMs} />
      </span>
    </div>
  )
}
