import { useQuery } from "@tanstack/react-query"
import { AGENT_NAMES } from "@/entities/agent"
import {
  agentAuthQuery,
  agentCheckQuery,
  agentSettingsQuery,
  gitStatusQuery,
  recorderStatusQuery,
  setupStatusQuery,
  trackerStatusQuery,
} from "@/shared/api/memorius-commands"

/** What setup checks, read again every few seconds while it is open. */
export function useSetupFacts() {
  const titleAccess = useQuery({
    ...recorderStatusQuery(),
    refetchInterval: 1_000,
    select: (recorder) => recorder.titleAccess,
  }).data
  const status = useQuery({
    ...setupStatusQuery(),
    refetchInterval: 1_000,
  }).data
  const trackerUrl = useQuery({
    ...trackerStatusQuery(),
    refetchInterval: 2_000,
    select: (tracker) => (tracker.hasToken && tracker.url ? tracker.url : null),
  }).data
  const agentAuth = useQuery({
    ...agentAuthQuery(),
    refetchInterval: 3_000,
  }).data
  const cliFound = useQuery({
    ...agentCheckQuery(),
    refetchInterval: 3_000,
    select: (check) => check.memoriusPath !== null,
  }).data
  const git = useQuery({ ...gitStatusQuery(), refetchInterval: 5_000 }).data
  const agentName =
    useQuery({
      ...agentSettingsQuery(),
      select: (settings) => AGENT_NAMES[settings.agent],
    }).data ?? "Your agent"
  return {
    titleAccess,
    status,
    trackerUrl,
    agentAuth,
    cliFound,
    git,
    agentName,
  }
}
