import type { AgentAuth } from "@/shared/api/generated/AgentAuth"
import type { SetupStatus } from "@/shared/api/generated/SetupStatus"
import type { TitleAccess } from "@/shared/api/generated/TitleAccess"

export type StepId =
  | "notifications"
  | "titles"
  | "login"
  | "repositories"
  | "agent"
  | "commandLine"
  | "tracker"

export type SetupStep = {
  id: StepId
  required: boolean
  done: boolean
}

export type SetupFacts = {
  titleAccess: TitleAccess | undefined
  status: SetupStatus | undefined
  agentAuth: AgentAuth | undefined
  /** The `memorius` command is on the PATH for the user's own sessions. */
  cliFound: boolean
  trackerConnected: boolean
  /** Repositories the last commits scan found. */
  repositories: number
}

/** The setup list in the order the user works through it. */
export function setupSteps(facts: SetupFacts): SetupStep[] {
  const agentReady =
    facts.agentAuth === "signedIn" || facts.agentAuth === "custom"
  return [
    {
      id: "notifications",
      required: false,
      done: facts.status?.notificationsSeen ?? false,
    },
    { id: "titles", required: true, done: facts.titleAccess === "granted" },
    {
      id: "login",
      required: false,
      done: facts.status?.startAtLogin ?? false,
    },
    { id: "repositories", required: false, done: facts.repositories > 0 },
    { id: "agent", required: false, done: agentReady },
    { id: "commandLine", required: false, done: facts.cliFound },
    { id: "tracker", required: false, done: facts.trackerConnected },
  ]
}

/** The step to work on: the first one neither done nor skipped. */
export function activeStep(
  steps: readonly SetupStep[],
  skipped: ReadonlySet<StepId>,
) {
  return steps.find((step) => !step.done && !skipped.has(step.id))?.id ?? null
}

export const requiredLeft = (steps: readonly SetupStep[]) =>
  steps.filter((step) => step.required && !step.done).length
