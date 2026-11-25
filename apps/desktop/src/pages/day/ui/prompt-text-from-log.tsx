import { useQuery } from "@tanstack/react-query"
import type { SessionRef } from "@/shared/api/generated/SessionRef"
import { promptTextQuery } from "@/shared/api/memorius-commands"
import { Skeleton } from "@/shared/ui/skeleton"

/** A prompt's text from its local session log, loaded while it is hovered. */
export function PromptTextFromLog({
  sessions,
  atMs,
}: {
  sessions: SessionRef[]
  atMs: number
}) {
  const text = useQuery(promptTextQuery(sessions, atMs))
  if (text.isPending) return <Skeleton className="h-3.5 w-48" />
  if (text.isError || !text.data) {
    return <span className="text-t4">Text no longer available</span>
  }
  return text.data
}
