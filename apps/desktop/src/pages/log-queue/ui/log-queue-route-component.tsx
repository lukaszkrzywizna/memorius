import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useNavigate } from "@tanstack/react-router"
import { useState } from "react"
import { openDates } from "@/pages/log-queue/lib/queue-rows"
import { useAgentRuns } from "@/pages/log-queue/model/use-agent-runs"
import {
  type BulkApproval,
  LogQueuePage,
} from "@/pages/log-queue/ui/log-queue-page"
import {
  agentSettingsQuery,
  approveProposal,
  daySummariesQuery,
  logApproved,
  logQueueQuery,
  memoriusQueryKey,
  rejectProposal,
  startAgentRun,
  stopAgentRun,
  trackerStatusQuery,
  trackerWorkTypesQuery,
  undoProposal,
} from "@/shared/api/memorius-commands"

export function LogQueueRouteComponent() {
  const queryClient = useQueryClient()
  const navigate = useNavigate()
  const runs = useAgentRuns()
  const running = runs[0]?.state.kind === "running"
  // Proposals show up while the agent makes them.
  const queue = useQuery({
    ...logQueueQuery(),
    refetchInterval: running ? 2_000 : false,
  })
  const settings = useQuery(agentSettingsQuery()).data
  // Decisions change the queue, the week and the day views.
  const refresh = () =>
    queryClient.invalidateQueries({ queryKey: memoriusQueryKey })
  const approve = useMutation({
    mutationFn: approveProposal,
    onSettled: refresh,
    onError: (error, { id }) =>
      setBulk({
        pending: false,
        failed: [{ id, issue: issueOf(id), message: String(error) }],
      }),
  })
  const reject = useMutation({ mutationFn: rejectProposal, onSuccess: refresh })
  const undo = useMutation({ mutationFn: undoProposal, onSuccess: refresh })
  const start = useMutation({ mutationFn: startAgentRun, onSettled: refresh })
  const stop = useMutation({ mutationFn: stopAgentRun, onSettled: refresh })
  // With a YouTrack token Memorius logs by itself; otherwise the agent does.
  const connected = useQuery(trackerStatusQuery()).data?.hasToken ?? false
  const log = useMutation({ mutationFn: logApproved, onSettled: refresh })
  const workTypes = useQuery(trackerWorkTypesQuery()).data ?? []
  const daySummaries =
    useQuery(daySummariesQuery(openDates(queue.data ?? []))).data ?? []
  const [bulk, setBulk] = useState<BulkApproval>({ pending: false, failed: [] })
  // What the user sees: an approval names the revision on screen.
  const shown = (id: number) =>
    queue.data?.find((item) => item.proposal.id === id)?.proposal
  const revisionOf = (id: number) => shown(id)?.revision ?? 0
  const issueOf = (id: number) => shown(id)?.issue ?? `#${id}`
  // One request per id: approved ones stay approved, failures are reported
  // and can be retried on their own.
  const approveMany = async (ids: number[]) => {
    setBulk({ pending: true, failed: [] })
    const results = await Promise.allSettled(
      ids.map((id) =>
        approveProposal({ id, revision: revisionOf(id), edit: null }),
      ),
    )
    await refresh()
    setBulk({
      pending: false,
      failed: results.flatMap((result, index) => {
        const id = ids[index]
        return result.status === "rejected" && id !== undefined
          ? [{ id, issue: issueOf(id), message: String(result.reason) }]
          : []
      }),
    })
  }

  return (
    <LogQueuePage
      items={queue.data}
      onApprove={(id, edit) =>
        approve.mutate({ id, revision: revisionOf(id), edit })
      }
      onReject={(id) => reject.mutate(id)}
      onUndo={(id) => undo.mutate(id)}
      onApproveMany={(ids) => void approveMany(ids)}
      bulk={bulk}
      workTypes={workTypes}
      daySummaries={daySummaries}
      agent={{
        settings,
        runs,
        onPrepare: (range) => start.mutate(range),
        onStop: () => stop.mutate(),
      }}
      youtrack={{
        onLog: (ids) =>
          connected ? log.mutate(ids) : void navigate({ to: "/settings" }),
        connected,
        pending: log.isPending,
        result: log.data,
        error: log.error?.toString(),
      }}
    />
  )
}
