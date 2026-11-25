import { useMutation, useQuery } from "@tanstack/react-query"
import { useNavigate } from "@tanstack/react-router"
import { AGENT_NAMES } from "@/entities/agent"
import { AttentionBanner } from "@/app/shell/ui/attention-banner"
import {
  agentAuthQuery,
  agentSettingsQuery,
  agentSignIn,
  recorderStatusQuery,
  setupStatusQuery,
} from "@/shared/api/memorius-commands"
import { formatClock } from "@/shared/lib/local-date"

/**
 * Banners for what stopped after setup: title access taken away, or the
 * agent signed out.
 */
export function SetupNotice() {
  const navigate = useNavigate()
  const done = useQuery(setupStatusQuery()).data?.done ?? false
  const status = useQuery(recorderStatusQuery()).data
  const agentAuth = useQuery({ ...agentAuthQuery(), enabled: done }).data
  const agent = useQuery(agentSettingsQuery()).data?.agent
  const signIn = useMutation({ mutationFn: agentSignIn })
  if (!done) return null
  return (
    <>
      {status?.titleAccess === "denied" && (
        <AttentionBanner
          tone="critical"
          title="Memorius can't read window titles."
          text={
            status.titlesLostMs === null
              ? "Only app names are recorded."
              : `Accessibility was switched off at ${formatClock(status.titlesLostMs)}; since then only app names are recorded.`
          }
          action="Fix"
          onAction={() => void navigate({ to: "/setup" })}
        />
      )}
      {agentAuth === "signedOut" && agent && (
        <AttentionBanner
          tone="attention"
          title={`${AGENT_NAMES[agent]} is signed out.`}
          text="The daily proposals run cannot start."
          action="Sign in again"
          onAction={() => signIn.mutate()}
        />
      )}
    </>
  )
}
