import type { ComponentProps } from "react"
import type { RecorderStatus } from "@/shared/api/generated/RecorderStatus"
import { AgentSection } from "@/pages/settings/ui/agent-section"
import {
  GitSection,
  type GitSectionProps,
} from "@/pages/settings/ui/git-section"
import {
  ReminderSection,
  type ReminderSectionProps,
} from "@/pages/settings/ui/reminder-section"
import {
  TrackerSection,
  type TrackerSectionProps,
} from "@/pages/settings/ui/tracker-section"
import {
  AdvancedSection,
  type AdvancedSectionProps,
} from "@/pages/settings/ui/advanced-section"
import {
  UpdatesSection,
  type UpdatesSectionProps,
} from "@/pages/settings/ui/updates-section"
import { formatClock } from "@/shared/lib/local-date"
import { Button } from "@/shared/ui/button"
import { SettingsSection } from "@/shared/ui/settings-section"

type SettingsPageProps = {
  status: RecorderStatus | undefined
  onRequestTitleAccess: () => void
  onPause: (minutes: number) => void
  onResume: () => void
  agent: ComponentProps<typeof AgentSection>
  tracker: TrackerSectionProps
  reminder: ReminderSectionProps
  git: GitSectionProps
  updates: UpdatesSectionProps
  advanced: AdvancedSectionProps
  /** Opens the setup list again. */
  onOpenSetup: () => void
}

export function SettingsPage({
  status,
  onRequestTitleAccess,
  onPause,
  onResume,
  agent,
  tracker,
  reminder,
  git,
  updates,
  advanced,
  onOpenSetup,
}: SettingsPageProps) {
  const denied = status?.titleAccess === "denied"
  return (
    <div className="flex max-w-[1080px] flex-col gap-1.5 px-1 pt-1.5 pb-6">
      <header className="flex flex-col gap-1.5 pb-3">
        <h1 className="m-0 text-[34px] leading-none font-light tracking-tight text-t1">
          Settings
        </h1>
        <p className="m-0 text-t4">
          How Memorius records on this computer and which agent it starts.
        </p>
      </header>
      <SettingsSection
        id="titles-heading"
        title="Window titles"
        description="Memorius reads the title of the window in front. It never records keystrokes, screenshots or document contents."
      >
        <div className="glass flex flex-col gap-3 rounded-2xl px-4 py-3.5 text-[13px]">
          <span className="flex items-center gap-2 text-t1">
            <span
              aria-hidden="true"
              className={`size-1.5 ${denied ? "bg-attention" : "bg-operational"}`}
            />
            {denied ? "Titles are hidden" : "Titles are recorded"}
          </span>
          {denied && (
            <>
              <p className="m-0 leading-normal text-t3">
                macOS hides window titles until you allow Memorius in System
                Settings → Privacy &amp; Security → Accessibility. Until then
                Memorius records apps, idle time and screen lock only.
              </p>
              <Button className="self-start" onClick={onRequestTitleAccess}>
                Allow window titles
              </Button>
            </>
          )}
        </div>
      </SettingsSection>
      <SettingsSection
        id="recording-heading"
        title="Recording"
        description="Pause when you do something you do not want on the timeline; recording starts again by itself."
      >
        <div className="glass flex flex-wrap items-center gap-2 rounded-2xl px-4 py-3 text-[13px]">
          {status?.state.kind === "paused" ? (
            <>
              <span className="mr-auto text-t1">
                Paused until {formatClock(status.state.untilMs)}
              </span>
              <Button onClick={onResume}>Resume now</Button>
            </>
          ) : (
            <>
              <span className="mr-auto flex items-center gap-2 text-t1">
                <span aria-hidden="true" className="size-1.5 bg-operational" />
                Recording is on
              </span>
              <Button variant="outline" onClick={() => onPause(30)}>
                Pause for 30 minutes
              </Button>
              <Button variant="outline" onClick={() => onPause(60)}>
                Pause for 1 hour
              </Button>
            </>
          )}
        </div>
      </SettingsSection>
      <ReminderSection {...reminder} />
      <TrackerSection {...tracker} />
      <GitSection {...git} />
      <UpdatesSection {...updates} />
      <AdvancedSection {...advanced} />
      <SettingsSection
        id="setup-heading"
        title="Setup"
        description="The permissions and connections from the first launch, with what is still off."
      >
        <div>
          <Button size="sm" variant="outline" onClick={onOpenSetup}>
            Open setup
          </Button>
        </div>
      </SettingsSection>
      <AgentSection {...agent} />
    </div>
  )
}
