import { XIcon } from "lucide-react"
import { useState } from "react"
import type { AppUsage } from "@/shared/api/generated/AppUsage"
import type { Rule } from "@/shared/api/generated/Rule"
import type { RuleKind } from "@/shared/api/generated/RuleKind"
import { AppIcon } from "@/entities/activity"
import { SettingsSection } from "@/shared/ui/settings-section"
import { formatDuration } from "@/shared/lib/duration"
import { Button } from "@/shared/ui/button"
import { NativeSelect, NativeSelectOption } from "@/shared/ui/native-select"

type PrivateAppsSectionProps = {
  rules: readonly Rule[]
  apps: readonly AppUsage[]
  appName: (appId: string) => string
  onAdd: (rule: RuleKind) => void
  onRemove: (id: number) => void
}

export function PrivateAppsSection({
  rules,
  apps,
  appName,
  onAdd,
  onRemove,
}: PrivateAppsSectionProps) {
  const privateRules = rules.flatMap((rule) =>
    rule.rule.kind === "privateApp"
      ? [{ id: rule.id, appId: rule.rule.appId }]
      : [],
  )
  const privateIds = new Set(privateRules.map((rule) => rule.appId))
  const candidates = apps.filter((usage) => !privateIds.has(usage.app.id))
  const [appId, setAppId] = useState("")
  const chosen = candidates.some((usage) => usage.app.id === appId) ? appId : ""
  const usage = (id: string) => apps.find((app) => app.app.id === id)

  return (
    <SettingsSection
      id="private-heading"
      title="Private apps"
      description="Only the app name and time are kept: no window titles, documents or addresses. Private apps still count as work time."
    >
      <div className="glass overflow-hidden rounded-2xl">
        <ul className="m-0 list-none p-0">
          {privateRules.map((rule) => {
            const recent = usage(rule.appId)
            const name = appName(rule.appId)
            return (
              <li
                key={rule.id}
                className="grid h-10.5 grid-cols-[20px_minmax(0,1fr)_auto_28px] items-center gap-2.5 border-b border-border pr-2 pl-3.5 text-[13px]"
              >
                <AppIcon app={{ id: rule.appId, name }} size={18} />
                <span className="truncate text-t1">{name}</span>
                <span className="text-xs whitespace-nowrap text-t4">
                  {recent
                    ? `${formatDuration(recent.durationMs)} in 14 days`
                    : "not used in 14 days"}
                </span>
                <button
                  type="button"
                  aria-label={`Remove ${name} from private apps`}
                  title="Remove"
                  onClick={() => onRemove(rule.id)}
                  className="grid size-7 place-items-center rounded-full text-t3 outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
                >
                  <XIcon aria-hidden="true" className="size-3" />
                </button>
              </li>
            )
          })}
        </ul>
        <div className="flex items-center gap-2 px-3.5 py-2">
          <NativeSelect
            aria-label="App used in the last 14 days"
            className="w-full"
            value={chosen}
            onChange={(event) => setAppId(event.target.value)}
          >
            <NativeSelectOption value="">
              Add an app used in the last 14 days…
            </NativeSelectOption>
            {candidates.map((candidate) => (
              <NativeSelectOption
                key={candidate.app.id}
                value={candidate.app.id}
              >
                {candidate.app.name} · {formatDuration(candidate.durationMs)}
              </NativeSelectOption>
            ))}
          </NativeSelect>
          <Button
            disabled={chosen === ""}
            onClick={() => {
              onAdd({ kind: "privateApp", appId: chosen })
              setAppId("")
            }}
          >
            Make private
          </Button>
        </div>
      </div>
    </SettingsSection>
  )
}
