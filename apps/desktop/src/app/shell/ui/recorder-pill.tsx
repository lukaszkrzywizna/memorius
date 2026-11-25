import type { ReactElement } from "react"
import type { RecorderStatus } from "@/shared/api/generated/RecorderStatus"
import { recorderDetail, recorderState } from "@/app/shell/lib/recorder-text"
import { Tooltip, TooltipContent, TooltipTrigger } from "@/shared/ui/tooltip"

type RecorderPillProps = {
  status: RecorderStatus | undefined
  /** The link to the recorder settings that the pill renders as. */
  link: ReactElement
}

/**
 * The recorder's state and the app in front, in the top bar. The live
 * region speaks only state changes; the window title stays in the tooltip,
 * so the polling does not flood a screen reader.
 */
export function RecorderPill({ status, link }: RecorderPillProps) {
  const state = recorderState(status)
  const tone =
    state.kind === "titlesHidden"
      ? "bg-attention"
      : state.kind === "recording"
        ? "bg-operational"
        : "bg-t4"
  const app =
    status?.state.kind === "recording" ? status.state.current?.app.name : null
  return (
    <>
      <span role="status" className="sr-only">
        {state.text}
      </span>
      <Tooltip>
        <TooltipTrigger
          render={link}
          aria-label={`${state.text}. Recorder settings`}
          className="glass inline-flex h-10 items-center gap-2 rounded-full px-4 text-t3 outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring max-[960px]:px-3.5"
        >
          <span aria-hidden="true" className={`size-1.5 ${tone}`} />
          <span className="max-[960px]:hidden">
            {app ? `${state.text} · ${app}` : state.text}
          </span>
          <span className="hidden max-[960px]:inline">{app ?? state.text}</span>
        </TooltipTrigger>
        <TooltipContent side="bottom">{recorderDetail(status)}</TooltipContent>
      </Tooltip>
    </>
  )
}
