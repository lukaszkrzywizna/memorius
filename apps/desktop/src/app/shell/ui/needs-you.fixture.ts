import type { NeedsYou } from "@/app/shell/model/needs-you"
import type { Proposal } from "@/shared/api/generated/Proposal"

const at = (clock: string) => {
  const [hours = 0, minutes = 0] = clock.split(":").map(Number)
  return new Date(2026, 9, 9, hours, minutes).getTime()
}

const proposal = (
  id: number,
  issue: string,
  minutes: number,
  text: string,
): Proposal => ({
  id,
  issue,
  date: "2026-10-09",
  startMs: at("13:31"),
  endMs: at("13:31") + minutes * 60_000,
  minutes,
  workType: "Development",
  text,
  agent: "Claude Code",
  createdMs: at("17:00"),
  state: { kind: "proposed" },
  lastError: null,
  revision: 1,
})

export const sampleNeeds: NeedsYou = {
  waiting: [
    proposal(
      1,
      "SEL-54",
      79,
      "Moved the customer history dashboard to the app stack",
    ),
    proposal(
      2,
      "IS-61",
      40,
      "Compared the north and south store results against production",
    ),
  ],
  unlogged: [
    { issue: "IS-61", minutes: 107 },
    { issue: "SEL-54", minutes: 45 },
    { issue: "INT-290", minutes: 20 },
  ],
  unloggedMs: 172 * 60_000,
  count: 2,
}

export const nothingNeeded: NeedsYou = {
  waiting: [],
  unlogged: [],
  unloggedMs: 0,
  count: 0,
}
