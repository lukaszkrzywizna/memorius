import { useState } from "react"
import type { TrackerStatus } from "@/shared/api/generated/TrackerStatus"
import { formatClock, formatShortDate } from "@/shared/lib/local-date"
import { Button } from "@/shared/ui/button"
import { Input } from "@/shared/ui/input"
import { SettingsSection } from "@/shared/ui/settings-section"

export type TrackerSectionProps = {
  status: TrackerStatus | undefined
  /** The connect or sync request in flight, and its outcome. */
  busy: boolean
  connectedAs: string | undefined
  error: string | undefined
  onConnect: (url: string, token: string | null) => void
  onSync: () => void
  onDisconnect: () => void
}

/** The YouTrack address and token, and how the last sync went. */
export function TrackerSection({
  status,
  busy,
  connectedAs,
  error,
  onConnect,
  onSync,
  onDisconnect,
}: TrackerSectionProps) {
  const [url, setUrl] = useState<string | null>(null)
  const [token, setToken] = useState("")
  const address = url ?? status?.url ?? ""
  const sync = status?.sync
  return (
    <SettingsSection
      id="tracker-heading"
      title="YouTrack"
      description="Memorius reads your own work items from the last 90 days, so Week shows what is really logged and the agent does not propose it again. The token is encrypted in Memorius's vault, whose key stays in the macOS keychain."
    >
      {status && (
        <div className="glass flex flex-col gap-3 rounded-2xl px-4 py-3.5 text-[13px]">
          <div className="flex flex-col gap-1.5 text-t4">
            <label htmlFor="tracker-url">Address</label>
            <Input
              id="tracker-url"
              placeholder="https://example.youtrack.cloud"
              value={address}
              onChange={(event) => setUrl(event.target.value)}
            />
          </div>
          <div className="flex flex-col gap-1.5 text-t4">
            <label htmlFor="tracker-token">Permanent token</label>
            <Input
              id="tracker-token"
              type="password"
              autoComplete="off"
              placeholder={
                status.hasToken
                  ? "Saved in the vault; paste a new one to replace it"
                  : "perm:…"
              }
              value={token}
              onChange={(event) => setToken(event.target.value)}
            />
            <span className="text-xs">
              YouTrack → Profile → Account Security → New token, scope YouTrack.
            </span>
          </div>
          <span className="flex items-center gap-2 text-t2">
            <span
              aria-hidden="true"
              className={`size-1.5 shrink-0 ${
                error || sync?.kind === "failed"
                  ? "bg-critical"
                  : sync?.kind === "synced"
                    ? "bg-operational"
                    : "bg-t5"
              }`}
            />
            {error ??
              (connectedAs
                ? `Connected as ${connectedAs}`
                : sync?.kind === "synced"
                  ? `Synced ${formatShortDate(sync.atMs)} ${formatClock(sync.atMs)} · ${sync.items} work items`
                  : sync?.kind === "failed"
                    ? `Sync failed ${formatClock(sync.atMs)}: ${sync.error}`
                    : "Not connected")}
          </span>
          <div className="flex flex-wrap gap-2">
            <Button
              disabled={busy || !address || (!token && !status.hasToken)}
              onClick={() => {
                onConnect(address, token || null)
                setToken("")
              }}
            >
              {busy ? "Checking…" : "Connect"}
            </Button>
            {status.hasToken && (
              <>
                <Button variant="outline" disabled={busy} onClick={onSync}>
                  Sync now
                </Button>
                <Button variant="ghost" disabled={busy} onClick={onDisconnect}>
                  Remove token
                </Button>
              </>
            )}
          </div>
        </div>
      )}
    </SettingsSection>
  )
}
