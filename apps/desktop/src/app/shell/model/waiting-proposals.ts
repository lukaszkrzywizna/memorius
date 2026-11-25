import { useQuery } from "@tanstack/react-query"
import { logQueueQuery } from "@/shared/api/memorius-commands"

/** Proposals that wait for the user's answer. */
export function useWaitingProposals() {
  const queue = useQuery(logQueueQuery())
  return (
    queue.data?.filter((item) => item.proposal.state.kind === "proposed")
      .length ?? 0
  )
}
