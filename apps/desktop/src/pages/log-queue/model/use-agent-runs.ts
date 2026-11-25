import { useQuery, useQueryClient } from "@tanstack/react-query"
import { useEffect } from "react"
import {
  agentRunsQuery,
  memoriusQueryKey,
} from "@/shared/api/memorius-commands"

/** The newest agent runs; when one ends, everything it may have changed reloads. */
export function useAgentRuns() {
  const queryClient = useQueryClient()
  const runs = useQuery(agentRunsQuery()).data ?? []
  const latest = runs[0]
  const finished =
    latest && latest.state.kind !== "running" ? latest.id : undefined
  useEffect(
    function reloadAfterRun() {
      if (finished !== undefined)
        void queryClient.invalidateQueries({ queryKey: memoriusQueryKey })
    },
    [finished, queryClient],
  )
  return runs
}
