import type { ReminderSettings } from "@/shared/api/generated/ReminderSettings"
import { Button } from "@/shared/ui/button"
import { Input } from "@/shared/ui/input"
import { SettingsSection } from "@/shared/ui/settings-section"

export type ReminderSectionProps = {
  settings: ReminderSettings | undefined
  onChange: (settings: ReminderSettings) => void
  onTest: () => void
  testing: boolean
  error: string | undefined
}

/** A daily notification when this week has time that is not logged. */
export function ReminderSection({
  settings,
  onChange,
  onTest,
  testing,
  error,
}: ReminderSectionProps) {
  return (
    <SettingsSection
      id="reminder-heading"
      title="Reminder"
      description="Once a day Memorius tells you how much of this week is not logged or proposed yet. It stays quiet when less than 15 minutes are left."
    >
      {settings && (
        <div className="glass flex flex-col gap-3 rounded-2xl px-4 py-3.5 text-[13px]">
          <div className="flex flex-wrap items-center gap-2">
            <label className="flex items-center gap-2 text-t2">
              <input
                type="checkbox"
                checked={settings.enabled}
                onChange={(event) =>
                  onChange({ ...settings, enabled: event.target.checked })
                }
              />
              Remind me every day at
            </label>
            <Input
              type="time"
              aria-label="Reminder time"
              className="w-28"
              value={settings.time}
              disabled={!settings.enabled}
              onChange={(event) =>
                onChange({ ...settings, time: event.target.value })
              }
            />
            <Button
              variant="outline"
              className="ml-auto"
              disabled={testing}
              onClick={onTest}
            >
              {testing ? "Sending…" : "Send a test"}
            </Button>
          </div>
          {error && <p className="m-0 text-critical">{error}</p>}
        </div>
      )}
    </SettingsSection>
  )
}
