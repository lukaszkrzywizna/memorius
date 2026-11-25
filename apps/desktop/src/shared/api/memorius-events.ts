import { listen } from "@tauri-apps/api/event"

/** Emitted by the recorder when a new activity row starts or recording state changes. */
const ACTIVITY_CHANGED = "activity-changed"

export function onActivityChanged(handler: () => void) {
  return listen(ACTIVITY_CHANGED, handler)
}
