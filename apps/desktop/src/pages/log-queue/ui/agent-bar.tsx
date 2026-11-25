import {
  CalendarRangeIcon,
  ChevronDownIcon,
  SparklesIcon,
  SquareIcon,
} from "lucide-react"
import { useState } from "react"
import type { AgentRun } from "@/shared/api/generated/AgentRun"
import type { AgentSettings } from "@/shared/api/generated/AgentSettings"
import type { RunRange } from "@/shared/api/generated/RunRange"
import { AppIcon } from "@/entities/activity"
import { AGENT_NAMES, RANGE_LABELS, runDays } from "@/entities/agent"
import { proposerApp } from "@/pages/log-queue/lib/queue-rows"
import { formatClock } from "@/shared/lib/local-date"
import { Button } from "@/shared/ui/button"
import { NativeSelect } from "@/shared/ui/native-select"

type AgentBarProps = {
  settings: AgentSettings | undefined
  latest: AgentRun | undefined
  onPrepare: (range: RunRange) => void
  onStop: () => void
}

/** Above this the summary folds to four lines behind "Show more". */
const LONG_SUMMARY = 360 // A character count; measure the overflow if it misjudges.

/**
 * The agent card: who, when and how the last run went with the controls on
 * top, and the agent's own summary below as a paragraph.
 */
export function AgentBar({
  settings,
  latest,
  onPrepare,
  onStop,
}: AgentBarProps) {
  const [expanded, setExpanded] = useState(false)
  const agent = latest?.agent ?? settings?.agent
  const name = agent ? AGENT_NAMES[agent] : ""
  const state = latest?.state
  const { meta, text, tone, running } = runView(latest, settings)
  const long = text.length > LONG_SUMMARY
  return (
    <section
      aria-label="Agent"
      className="glass flex flex-col gap-2 rounded-2xl px-3.5 py-3 text-[13px]"
    >
      <div className="flex flex-wrap items-center gap-x-2.5 gap-y-2">
        {agent && (
          <span className="flex items-center gap-2 font-medium text-t1">
            <AppIcon app={proposerApp(name)} size={20} />
            {name}
          </span>
        )}
        {meta && <span className="text-t3">{meta}</span>}
        {state && <RunChip state={state} />}
        <span className="ml-auto flex shrink-0 items-center gap-2">
          {running ? (
            <Button size="sm" variant="outline" onClick={onStop}>
              <SquareIcon aria-hidden="true" className="fill-current" />
              Stop
            </Button>
          ) : (
            <PrepareControls settings={settings} onPrepare={onPrepare} />
          )}
        </span>
      </div>
      {text && (
        <div className="flex flex-col items-start gap-1">
          <p
            className={`m-0 max-w-[1100px] leading-normal break-words ${tone} ${
              long && !expanded ? "line-clamp-4" : ""
            }`}
          >
            {text}
          </p>
          {long && (
            <button
              type="button"
              aria-expanded={expanded}
              onClick={() => setExpanded(!expanded)}
              className="inline-flex items-center gap-1 rounded-full text-xs text-t3 outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
            >
              {expanded ? "Show less" : "Show more"}
              <ChevronDownIcon
                aria-hidden="true"
                className={`size-3.5 ${expanded ? "rotate-180" : ""}`}
              />
            </button>
          )}
        </div>
      )}
    </section>
  )
}

/** What the card says about the last run, or the schedule before one. */
function runView(
  latest: AgentRun | undefined,
  settings: AgentSettings | undefined,
) {
  if (!latest) {
    const meta = settings
      ? `prepares proposals ${settings.daily ? `every day at ${settings.dailyTime}` : "when you ask"}`
      : ""
    return { meta, text: "", tone: "text-t2", running: false }
  }
  const meta = (when: string) =>
    [when, runDays(latest)].filter(Boolean).join(" · ")
  switch (latest.state.kind) {
    case "running":
      return {
        meta: meta(`started ${formatClock(latest.startedMs)}`),
        text: "Reading your days and YouTrack worklogs. Proposals appear in the queue when the run ends.",
        tone: "text-t3",
        running: true,
      }
    case "done":
      return {
        meta: meta(`last run ${formatClock(latest.state.finishedMs)}`),
        text: latest.state.summary || "Done",
        tone: "text-t2",
        running: false,
      }
    case "failed":
      return {
        meta: meta(`last run ${formatClock(latest.state.finishedMs)}`),
        text: latest.state.error,
        tone: "text-critical",
        running: false,
      }
  }
}

const CHIPS = {
  running: [
    "Preparing proposals",
    "bg-attention/15 text-t1",
    "animate-pulse bg-attention",
  ],
  done: ["Done", "bg-operational/15 text-t1", "bg-operational"],
  failed: ["Failed", "bg-critical/15 text-t1", "bg-critical"],
} as const

function RunChip({ state }: { state: AgentRun["state"] }) {
  const [label, chip, dot] = CHIPS[state.kind]
  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-full px-2 py-0.5 text-xs ${chip}`}
    >
      <span aria-hidden="true" className={`size-1.5 rounded-full ${dot}`} />
      {label}
    </span>
  )
}

/** The range to cover and the button that starts a proposals run. */
export function PrepareControls({
  settings,
  onPrepare,
}: {
  settings: AgentSettings | undefined
  onPrepare: (range: RunRange) => void
}) {
  const [range, setRange] = useState<RunRange | null>(null)
  const chosen = range ?? settings?.defaultRange ?? "sinceLastRun"
  return (
    <>
      <span className="flex items-center gap-1.5 whitespace-nowrap text-t3">
        <CalendarRangeIcon aria-hidden="true" className="size-4" />
        <NativeSelect
          size="sm"
          aria-label="Range"
          value={chosen}
          onChange={(event) => setRange(event.target.value as RunRange)}
        >
          {Object.entries(RANGE_LABELS).map(([value, label]) => (
            <option key={value} value={value}>
              {label}
            </option>
          ))}
        </NativeSelect>
      </span>
      <Button size="sm" disabled={!settings} onClick={() => onPrepare(chosen)}>
        <SparklesIcon aria-hidden="true" />
        Prepare proposals
      </Button>
    </>
  )
}
