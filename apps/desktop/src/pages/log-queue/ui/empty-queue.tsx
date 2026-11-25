import { ListChecksIcon, LoaderCircleIcon, SquareIcon } from "lucide-react"
import type { AgentRun } from "@/shared/api/generated/AgentRun"
import type { AgentSettings } from "@/shared/api/generated/AgentSettings"
import type { RunRange } from "@/shared/api/generated/RunRange"
import { AGENT_NAMES, runText } from "@/entities/agent"
import { PrepareControls } from "@/pages/log-queue/ui/agent-bar"
import { Button } from "@/shared/ui/button"

type EmptyQueueProps = {
  settings: AgentSettings | undefined
  latest: AgentRun | undefined
  onPrepare: (range: RunRange) => void
  onStop: () => void
}

/** An empty queue: one card that starts the agent, or shows it working. */
export function EmptyQueue({
  settings,
  latest,
  onPrepare,
  onStop,
}: EmptyQueueProps) {
  const running = latest?.state.kind === "running"
  const name = settings ? AGENT_NAMES[settings.agent] : "Your agent"
  return (
    <section
      aria-label="Agent"
      className="glass mx-auto mt-[10vh] flex w-full max-w-[440px] flex-col items-center gap-3 rounded-[24px] px-8 py-9 text-center"
    >
      <span className="mb-1 grid size-14 place-items-center rounded-full bg-muted text-t2">
        {running ? (
          <LoaderCircleIcon
            aria-hidden="true"
            className="size-6 animate-spin"
          />
        ) : (
          <ListChecksIcon aria-hidden="true" className="size-6" />
        )}
      </span>
      <h2 className="m-0 text-[22px] font-light text-t1">
        {running ? "Preparing proposals" : "No proposals yet"}
      </h2>
      <p className="m-0 text-[13px] leading-normal text-t3">
        {running && latest
          ? runText(latest)
          : `${name} reads your activity and proposes worklogs. You approve each one here before anything goes to YouTrack.`}
      </p>
      {latest && !running && (
        <p
          className={`m-0 line-clamp-3 text-xs leading-normal ${
            latest.state.kind === "failed" ? "text-critical" : "text-t4"
          }`}
          title={runText(latest)}
        >
          Last run {runText(latest)}
        </p>
      )}
      <div className="mt-2 flex flex-wrap justify-center gap-2">
        {running ? (
          <Button size="sm" variant="outline" onClick={onStop}>
            <SquareIcon aria-hidden="true" className="fill-current" />
            Stop
          </Button>
        ) : (
          <PrepareControls settings={settings} onPrepare={onPrepare} />
        )}
      </div>
    </section>
  )
}
