import {
  BellIcon,
  CheckIcon,
  GitBranchIcon,
  LinkIcon,
  type LucideIcon,
  PanelTopIcon,
  PowerIcon,
  SparklesIcon,
  TerminalIcon,
} from "lucide-react"
import { type ReactNode, useState } from "react"
import {
  type SetupStep,
  type StepId,
  activeStep,
  requiredLeft,
} from "@/pages/setup/lib/setup-steps"
import { Button } from "@/shared/ui/button"
import { Input } from "@/shared/ui/input"

type StepAction = { label: string; onClick: () => void }

export type SetupPageProps = {
  steps: readonly SetupStep[]
  /** What Memorius sees now, per step, for example "Signed out". */
  notes: Partial<Record<StepId, string>>
  /** The buttons of a step that is not done yet, main one first. */
  actions: Record<StepId, readonly StepAction[]>
  /** The step whose switch Memorius waits for after opening its settings. */
  waiting: StepId | null
  /** Folders to search for repositories, typed or picked from what was found. */
  folders: { suggestions: readonly string[]; onAdd: (folder: string) => void }
  onFinish: () => void
}

const STEPS: Record<StepId, { title: string; text: string; icon: LucideIcon }> =
  {
    notifications: {
      title: "Notifications",
      text: "A daily reminder about time not logged, and new versions.",
      icon: BellIcon,
    },
    titles: {
      title: "Window titles",
      text: "Accessibility lets Memorius read window titles and browser addresses. Nothing leaves this computer.",
      icon: PanelTopIcon,
    },
    login: {
      title: "Start at login",
      text: "So no hour of work is missed after a restart. Your system may ask you to allow it in its login items.",
      icon: PowerIcon,
    },
    repositories: {
      title: "Repositories",
      text: "Your own commits show next to your time. Memorius reads the git history of repositories in folders you choose and in folders you worked in.",
      icon: GitBranchIcon,
    },
    agent: {
      title: "Agent",
      text: "Your coding agent prepares worklog proposals through the memorius CLI.",
      icon: SparklesIcon,
    },
    commandLine: {
      title: "Command line",
      text: "For your own Claude Code or Codex sessions. Install command links memorius in ~/.local/bin; Add to … adds a memorius MCP server to that agent's user config and copies the memorius-worklogs skill.",
      icon: TerminalIcon,
    },
    tracker: {
      title: "YouTrack",
      text: "Shows the time you already logged and logs what you approve.",
      icon: LinkIcon,
    },
  }

/**
 * First-run setup: every permission and connection as a row that turns
 * green by itself once Memorius sees it; recording starts when the
 * required rows are done.
 */
export function SetupPage({
  steps,
  notes,
  actions,
  waiting,
  folders,
  onFinish,
}: SetupPageProps) {
  const [skipped, setSkipped] = useState<ReadonlySet<StepId>>(new Set())
  const active = activeStep(steps, skipped)
  const done = steps.filter((step) => step.done).length
  const left = requiredLeft(steps)
  return (
    <main className="grid min-h-dvh grid-cols-[300px_minmax(0,1fr)] gap-7 bg-(--backdrop) px-9 py-8 text-foreground max-[960px]:grid-cols-1">
      <div className="flex flex-col gap-3">
        <h1 className="m-0 text-[34px] leading-tight font-light tracking-tight text-t1">
          Set up Memorius
        </h1>
        <p className="m-0 text-[13px] leading-normal text-t3">
          Memorius records which apps and windows you work in, on this computer
          only. Only window titles are required; the rest is optional and can
          wait.
        </p>
        <div
          role="progressbar"
          aria-label="Setup progress"
          aria-valuemin={0}
          aria-valuemax={steps.length}
          aria-valuenow={done}
          className="mt-3 h-1.5 overflow-hidden rounded-full bg-well"
        >
          <span
            className="block h-full rounded-full bg-t1 transition-[width]"
            style={{ width: `${(done / steps.length) * 100}%` }}
          />
        </div>
        <span className="text-xs text-t4">
          {done} of {steps.length} done ·{" "}
          {left > 0 ? `${left} required left` : "nothing required left"}
        </span>
      </div>
      <div className="flex flex-col gap-4">
        <ol className="m-0 flex list-none flex-col gap-2 p-0">
          {steps.map((step) => (
            <StepRow
              key={step.id}
              step={step}
              note={notes[step.id]}
              actions={actions[step.id]}
              active={step.id === active}
              waiting={step.id === waiting}
              extra={
                step.id === "repositories" && !step.done ? (
                  <AddFolders {...folders} />
                ) : null
              }
              onSkip={() => setSkipped(new Set([...skipped, step.id]))}
            />
          ))}
        </ol>
        <div className="flex flex-wrap items-center justify-between gap-3">
          <span className="text-xs text-t4">
            You can come back to this list in Settings → Setup.
          </span>
          <Button size="lg" disabled={left > 0} onClick={onFinish}>
            Start recording
          </Button>
        </div>
      </div>
    </main>
  )
}

type StepRowProps = {
  step: SetupStep
  note: string | undefined
  actions: readonly StepAction[]
  active: boolean
  waiting: boolean
  /** Inputs that do the step right in its row. */
  extra: ReactNode
  onSkip: () => void
}

function StepRow({
  step,
  note,
  actions,
  active,
  waiting,
  extra,
  onSkip,
}: StepRowProps) {
  const { title, text, icon: Icon } = STEPS[step.id]
  return (
    <li
      aria-label={`${title}, ${step.done ? "done" : "to do"}`}
      aria-current={active ? "step" : undefined}
      className={`grid grid-cols-[32px_minmax(0,1fr)_auto] items-center gap-3.5 rounded-2xl border px-4 py-3.5 text-[13px] max-[720px]:grid-cols-[32px_minmax(0,1fr)] ${
        step.done
          ? "border-border bg-card/50"
          : active
            ? "border-foreground bg-card shadow-[0_0_0_3px_color-mix(in_srgb,var(--t1)_7%,transparent)]"
            : "border-border bg-card"
      }`}
    >
      <span
        aria-hidden="true"
        className={`grid size-8 place-items-center rounded-full ${
          step.done ? "bg-operational text-white" : "bg-well text-t3"
        }`}
      >
        {step.done ? (
          <CheckIcon className="size-4" strokeWidth={2.6} />
        ) : (
          <Icon className="size-4" />
        )}
      </span>
      <div className="flex min-w-0 flex-col gap-0.5">
        <span
          className={`flex items-center gap-2 font-medium ${step.done ? "text-t4" : "text-t1"}`}
        >
          {title}
          {!step.done && (
            <span
              className={`rounded-full px-1.5 text-[11px] font-normal ${
                step.required ? "bg-attention/20 text-t1" : "bg-well text-t3"
              }`}
            >
              {step.required ? "required" : "optional"}
            </span>
          )}
        </span>
        <span className={step.done ? "text-t4" : "text-t3"}>
          {note ?? text}
        </span>
        {extra}
        {waiting && !step.done && (
          <span
            role="status"
            className="mt-1 flex flex-col gap-1 text-xs text-t4"
          >
            <span className="flex items-center gap-1.5">
              <span
                aria-hidden="true"
                className="size-1.5 animate-pulse rounded-full bg-attention"
              />
              Waiting for Memorius to be switched on in System Settings…
            </span>
            <span>
              Already on and still waiting? The list can hold an older Memorius
              that only looks the same. Select every Memorius entry, remove it
              with −, then open System Settings again here and turn the new one
              on.
            </span>
          </span>
        )}
      </div>
      {!step.done && (
        <div className="flex flex-wrap items-center justify-end gap-2 max-[720px]:col-start-2 max-[720px]:justify-start">
          {!step.required && (
            <Button size="sm" variant="ghost" onClick={onSkip}>
              Skip
            </Button>
          )}
          {actions.map((action, index) => (
            <Button
              key={action.label}
              size="sm"
              variant={index === 0 && active ? "default" : "outline"}
              onClick={action.onClick}
            >
              {action.label}
            </Button>
          ))}
        </div>
      )}
    </li>
  )
}

/** A folder typed in, or one of the folders where repositories were found. */
function AddFolders({ suggestions, onAdd }: SetupPageProps["folders"]) {
  return (
    <div className="mt-2 flex flex-col gap-2">
      {suggestions.length > 0 && (
        <span className="flex flex-wrap items-center gap-2 text-xs text-t4">
          Found in
          {suggestions.map((folder) => (
            <Button
              key={folder}
              size="xs"
              variant="outline"
              onClick={() => onAdd(folder)}
            >
              <span className="font-mono">{folder}</span>
            </Button>
          ))}
        </span>
      )}
      <form
        className="flex max-w-96 gap-2"
        onSubmit={(event) => {
          event.preventDefault()
          const form = event.currentTarget
          const folder = new FormData(form).get("folder")
          if (typeof folder === "string" && folder.trim()) onAdd(folder.trim())
          form.reset()
        }}
      >
        <Input
          name="folder"
          aria-label="Repositories folder"
          className="h-8 font-mono"
          placeholder="~/code"
        />
        <Button type="submit" size="sm" variant="outline">
          Add
        </Button>
      </form>
    </div>
  )
}
