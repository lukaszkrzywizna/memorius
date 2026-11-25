import { describe, expect, it } from "vitest"
import {
  type SetupFacts,
  activeStep,
  requiredLeft,
  setupSteps,
} from "@/pages/setup/lib/setup-steps"

const facts: SetupFacts = {
  titleAccess: "denied",
  status: { done: false, startAtLogin: true, notificationsSeen: true },
  agentAuth: "signedOut",
  cliFound: true,
  trackerConnected: false,
  repositories: 3,
}

describe("setup steps", () => {
  it("works on the first open step and needs only title access", () => {
    const steps = setupSteps(facts)

    expect(activeStep(steps, new Set())).toBe("titles")
    expect(requiredLeft(steps)).toBe(1)
    const granted = setupSteps({ ...facts, titleAccess: "granted" })
    expect(requiredLeft(granted)).toBe(0)
    expect(activeStep(granted, new Set(["agent"]))).toBe("tracker")
  })

  it("counts the agent ready once signed in, the command line apart", () => {
    const step = (id: string, changes: Partial<SetupFacts>) =>
      setupSteps({ ...facts, ...changes }).find((s) => s.id === id)

    expect(
      step("agent", { agentAuth: "signedIn", cliFound: false })?.done,
    ).toBe(true)
    expect(step("commandLine", { cliFound: false })?.done).toBe(false)
  })
})
