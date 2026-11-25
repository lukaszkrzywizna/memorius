import type { TimeRules } from "@/shared/api/generated/TimeRules"
import { Button } from "@/shared/ui/button"
import { Input } from "@/shared/ui/input"
import { NativeSelect } from "@/shared/ui/native-select"
import { SettingsSection } from "@/shared/ui/settings-section"

export type AdvancedSectionProps = {
  rules: TimeRules | undefined
  onChange: (rules: TimeRules) => void
}

const DEFAULTS: TimeRules = {
  awayAfterMinutes: 5,
  breakMinutes: 15,
  agentAfterPromptMinutes: 30,
  weekStartsOn: "monday",
}

const FIELDS: readonly {
  key: "awayAfterMinutes" | "breakMinutes" | "agentAfterPromptMinutes"
  label: string
  hint: string
}[] = [
  {
    key: "awayAfterMinutes",
    label: "Away after",
    hint: "No keyboard or mouse input this long, and Memorius records you as away.",
  },
  {
    key: "breakMinutes",
    label: "Breaks counted as work",
    hint: "Pauses up to this long between moments at the computer are reading or thinking.",
  },
  {
    key: "agentAfterPromptMinutes",
    label: "Agent time after a prompt",
    hint: "How long an agent's work counts as yours after you prompt it; later it runs on its own.",
  },
]

/** The rules Memorius counts time with, for users who count differently. */
export function AdvancedSection({ rules, onChange }: AdvancedSectionProps) {
  return (
    <SettingsSection
      id="advanced-heading"
      title="Advanced"
      description="How Memorius turns activity into work time. Changes apply to every day, also past ones."
    >
      {rules && (
        <div className="glass flex flex-col gap-3 rounded-2xl px-4 py-3.5 text-[13px]">
          {FIELDS.map(({ key, label, hint }) => (
            <div key={key} className="flex flex-col gap-1 text-t4">
              <label htmlFor={key} className="text-t2">
                {label}
              </label>
              <span className="flex items-center gap-2">
                <Input
                  id={key}
                  key={rules[key]}
                  type="number"
                  min={key === "awayAfterMinutes" ? 1 : 0}
                  className="w-24"
                  defaultValue={rules[key]}
                  onBlur={(event) => {
                    const minutes = Number.parseInt(event.target.value, 10)
                    if (Number.isFinite(minutes) && minutes !== rules[key])
                      onChange({ ...rules, [key]: minutes })
                  }}
                />
                minutes
              </span>
              <span className="text-xs">{hint}</span>
            </div>
          ))}
          <div className="flex flex-col gap-1 text-t4">
            <label htmlFor="week-start" className="text-t2">
              Week starts on
            </label>
            <NativeSelect
              id="week-start"
              size="sm"
              value={rules.weekStartsOn}
              onChange={(event) =>
                onChange({
                  ...rules,
                  weekStartsOn:
                    event.target.value === "sunday" ? "sunday" : "monday",
                })
              }
            >
              <option value="monday">Monday</option>
              <option value="sunday">Sunday</option>
            </NativeSelect>
          </div>
          <div>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => onChange(DEFAULTS)}
            >
              Reset to defaults
            </Button>
          </div>
        </div>
      )}
    </SettingsSection>
  )
}
