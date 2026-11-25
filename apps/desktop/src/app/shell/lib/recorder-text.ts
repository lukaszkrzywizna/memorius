import type { RecorderStatus } from "@/shared/api/generated/RecorderStatus"
import { windowTitle } from "@/entities/activity"
import { formatClock } from "@/shared/lib/local-date"

type RecorderState = {
  kind: "starting" | "recording" | "titlesHidden" | "paused"
  text: string
}

/** The recorder's state in a few words; it changes only on a state change. */
export function recorderState(
  status: RecorderStatus | undefined,
): RecorderState {
  if (!status) return { kind: "starting", text: "Starting…" }
  switch (status.state.kind) {
    case "recording":
      return status.titleAccess === "denied"
        ? { kind: "titlesHidden", text: "Recording, titles hidden" }
        : { kind: "recording", text: "Recording" }
    case "paused":
      return {
        kind: "paused",
        text: `Paused until ${formatClock(status.state.untilMs)}`,
      }
  }
}

/** The state with the current app and window, for the tooltip. */
export function recorderDetail(status: RecorderStatus | undefined) {
  const state = recorderState(status)
  if (state.kind === "titlesHidden") return `${state.text}: allow Accessibility`
  if (status?.state.kind !== "recording") return state.text
  const current = status.state.current
  return current
    ? `${state.text} · ${current.app.name} · ${windowTitle(current.title)}`
    : `${state.text} · no app in front`
}
