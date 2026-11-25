import { useMutation, useQueryClient } from "@tanstack/react-query"
import { useNavigate } from "@tanstack/react-router"
import { useCommandLine } from "@/entities/agent"
import { setupSteps } from "@/pages/setup/lib/setup-steps"
import { useSetupFacts } from "@/pages/setup/model/use-setup-facts"
import { SetupPage } from "@/pages/setup/ui/setup-page"
import {
  agentSignIn,
  confirmNotifications,
  finishSetup,
  memoriusQueryKey,
  openSystemPane,
  requestTitleAccess,
  restartApp,
  saveGitSettings,
  startAtLogin,
  testReminder,
} from "@/shared/api/memorius-commands"
import { todayIso } from "@/shared/lib/local-date"

/** Checks every second while open, so rows turn green by themselves. */
export function SetupRouteComponent() {
  const queryClient = useQueryClient()
  const navigate = useNavigate()
  const {
    titleAccess,
    status,
    trackerUrl,
    agentAuth,
    cliFound,
    git,
    agentName,
  } = useSetupFacts()
  const commandLine = useCommandLine()
  const run = useMutation({
    mutationFn: (action: () => Promise<unknown>) => action(),
    onSettled: () =>
      queryClient.invalidateQueries({ queryKey: memoriusQueryKey }),
  })
  const titles = useMutation({ mutationFn: requestTitleAccess })
  const finish = useMutation({
    mutationFn: finishSetup,
    onSuccess: () =>
      navigate({ to: "/day/$date", params: { date: todayIso() } }),
  })

  const steps = setupSteps({
    titleAccess,
    status,
    agentAuth,
    cliFound: cliFound ?? false,
    trackerConnected: Boolean(trackerUrl),
    repositories: git?.scan?.repositories ?? 0,
  })
  const toSettings = () => void navigate({ to: "/settings" })
  const agentNote =
    agentAuth === "missing"
      ? `${agentName} is not installed.`
      : agentAuth === "signedOut"
        ? `${agentName} found, signed out.`
        : agentAuth === "signedIn" || agentAuth === "custom"
          ? `${agentName} is ready.`
          : undefined

  return (
    <SetupPage
      steps={steps}
      waiting={titles.isSuccess ? "titles" : null}
      notes={{
        notifications: status?.notificationsSeen ? "Allowed" : undefined,
        titles: titleAccess === "granted" ? "Allowed" : undefined,
        login: status?.startAtLogin ? "On" : undefined,
        repositories: git?.scan?.repositories
          ? `${git.scan.repositories} repositories found`
          : undefined,
        agent: agentNote,
        commandLine:
          commandLine.note ??
          (cliFound ? "memorius is on your PATH." : undefined),
        tracker: trackerUrl ? `Connected to ${trackerUrl}` : undefined,
      }}
      actions={{
        notifications: [
          { label: "Send a test", onClick: () => run.mutate(testReminder) },
          {
            label: "It arrived",
            onClick: () => run.mutate(confirmNotifications),
          },
        ],
        titles: [
          { label: "Open System Settings", onClick: () => titles.mutate() },
          ...(titles.isSuccess
            ? [{ label: "Restart Memorius", onClick: () => void restartApp() }]
            : []),
        ],
        login: [
          { label: "Turn on", onClick: () => run.mutate(startAtLogin) },
          {
            label: "Open Login Items",
            onClick: () => run.mutate(() => openSystemPane("loginItems")),
          },
        ],
        agent:
          agentAuth === "signedOut"
            ? [{ label: "Sign in", onClick: () => run.mutate(agentSignIn) }]
            : agentAuth === "missing"
              ? [{ label: "Choose an agent", onClick: toSettings }]
              : [],
        commandLine: [
          { label: "Install command", onClick: commandLine.onInstall },
          {
            label: "Add to Claude Code",
            onClick: () => commandLine.onAdd("claudeCode"),
          },
          { label: "Add to Codex", onClick: () => commandLine.onAdd("codex") },
        ],
        repositories: [],
        tracker: [{ label: "Connect", onClick: toSettings }],
      }}
      folders={{
        suggestions: git?.scan?.suggestions ?? [],
        onAdd: (folder) =>
          run.mutate(() =>
            saveGitSettings({
              roots: [...new Set([...(git?.settings.roots ?? []), folder])],
            }),
          ),
      }}
      onFinish={() => finish.mutate()}
    />
  )
}
