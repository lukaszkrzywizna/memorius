import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useNavigate } from "@tanstack/react-router"
import { useCommandLine } from "@/entities/agent"
import { SettingsPage } from "@/pages/settings/ui/settings-page"
import {
  agentCheckQuery,
  agentPromptQuery,
  connectTracker,
  gitStatusQuery,
  installUpdate,
  saveGitSettings,
  disconnectTracker,
  agentSettingsQuery,
  memoriusQueryKey,
  pauseRecording,
  recorderStatusQuery,
  reminderSettingsQuery,
  requestTitleAccess,
  resumeRecording,
  saveAgentSettings,
  saveReminderSettings,
  syncTracker,
  testReminder,
  trackerStatusQuery,
  updateStatusQuery,
  saveTimeRules,
  timeRulesQuery,
} from "@/shared/api/memorius-commands"

export function SettingsRouteComponent() {
  const queryClient = useQueryClient()
  const navigate = useNavigate()
  const status = useQuery(recorderStatusQuery())
  const refresh = () =>
    queryClient.invalidateQueries({ queryKey: memoriusQueryKey })
  const pause = useMutation({ mutationFn: pauseRecording, onSuccess: refresh })
  const resume = useMutation({
    mutationFn: resumeRecording,
    onSuccess: refresh,
  })
  const request = useMutation({ mutationFn: requestTitleAccess })
  const agentSettings = useQuery(agentSettingsQuery()).data
  const agentCheck = useQuery(agentCheckQuery()).data
  const agentPrompt = useQuery(agentPromptQuery()).data
  const gitStatus = useQuery(gitStatusQuery()).data
  const saveGit = useMutation({
    mutationFn: saveGitSettings,
    onSettled: refresh,
  })
  const reminderSettings = useQuery(reminderSettingsQuery()).data
  const saveReminder = useMutation({
    mutationFn: saveReminderSettings,
    onSuccess: refresh,
  })
  const test = useMutation({ mutationFn: testReminder })
  const trackerStatus = useQuery(trackerStatusQuery()).data
  const connect = useMutation({
    mutationFn: connectTracker,
    onSettled: refresh,
  })
  const sync = useMutation({ mutationFn: syncTracker, onSettled: refresh })
  const disconnect = useMutation({
    mutationFn: disconnectTracker,
    onSettled: refresh,
  })
  const timeRules = useQuery(timeRulesQuery()).data
  const saveRules = useMutation({
    mutationFn: saveTimeRules,
    onSuccess: refresh,
  })
  const updateStatus = useQuery(updateStatusQuery())
  const install = useMutation({ mutationFn: installUpdate })
  const commandLine = useCommandLine()
  const saveAgent = useMutation({
    mutationFn: saveAgentSettings,
    onSuccess: refresh,
  })

  return (
    <SettingsPage
      status={status.data}
      onRequestTitleAccess={() => request.mutate()}
      onPause={(minutes) => pause.mutate(minutes)}
      onResume={() => resume.mutate()}
      onOpenSetup={() => void navigate({ to: "/setup" })}
      advanced={{
        rules: timeRules,
        onChange: (rules) => saveRules.mutate(rules),
      }}
      updates={{
        status: updateStatus.data,
        checking: updateStatus.isFetching,
        installing: install.isPending,
        error: install.error?.toString(),
        onCheck: () => void updateStatus.refetch(),
        onInstall: () => install.mutate(),
      }}
      git={{
        status: gitStatus,
        saving: saveGit.isPending,
        error: saveGit.error?.toString(),
        onRootsChange: (roots) => saveGit.mutate({ roots }),
      }}
      reminder={{
        settings: reminderSettings,
        onChange: (settings) => saveReminder.mutate(settings),
        onTest: () => test.mutate(),
        testing: test.isPending,
        error: [saveReminder.error, test.error].find(Boolean)?.toString(),
      }}
      tracker={{
        status: trackerStatus,
        busy: connect.isPending || sync.isPending,
        connectedAs: connect.data,
        error: [connect.error, sync.error, disconnect.error]
          .find(Boolean)
          ?.toString(),
        onConnect: (url, token) => connect.mutate({ url, token }),
        onSync: () => sync.mutate(),
        onDisconnect: () => disconnect.mutate(),
      }}
      agent={{
        settings: agentSettings,
        check: agentCheck,
        prompt: agentPrompt,
        onChange: (settings) => saveAgent.mutate(settings),
        commandLine,
      }}
    />
  )
}
