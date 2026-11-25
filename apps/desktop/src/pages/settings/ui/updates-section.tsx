import { DownloadIcon, RefreshCwIcon } from "lucide-react"
import type { UpdateStatus } from "@/shared/api/generated/UpdateStatus"
import { Button } from "@/shared/ui/button"
import { SettingsSection } from "@/shared/ui/settings-section"

export type UpdatesSectionProps = {
  status: UpdateStatus | undefined
  checking: boolean
  installing: boolean
  error: string | undefined
  onCheck: () => void
  onInstall: () => void
}

/** The installed version and the newest release on GitHub. */
export function UpdatesSection({
  status,
  checking,
  installing,
  error,
  onCheck,
  onInstall,
}: UpdatesSectionProps) {
  const available = status?.available
  return (
    <SettingsSection
      id="updates-heading"
      title="Updates"
      description="Memorius checks GitHub for a new release once a day and tells you when one is ready. Updates keep your permissions and data."
    >
      <div className="glass flex flex-wrap items-center gap-3 rounded-2xl px-4 py-3.5 text-[13px]">
        <div className="mr-auto flex flex-col gap-0.5">
          <span className="text-t1">
            Memorius {status ? status.current : ""}
          </span>
          <span className={error ? "text-critical" : "text-t3"}>
            {error
              ? `Could not update: ${error}`
              : checking
                ? "Checking…"
                : available
                  ? `Version ${available} is available.`
                  : status?.problem
                    ? `${status.problem}.`
                    : status
                      ? "You have the latest version."
                      : ""}
          </span>
        </div>
        {available ? (
          <Button size="sm" disabled={installing} onClick={onInstall}>
            <DownloadIcon aria-hidden="true" />
            {installing ? "Installing…" : "Install and restart"}
          </Button>
        ) : (
          <Button
            size="sm"
            variant="outline"
            disabled={checking}
            onClick={onCheck}
          >
            <RefreshCwIcon aria-hidden="true" />
            Check now
          </Button>
        )}
      </div>
    </SettingsSection>
  )
}
