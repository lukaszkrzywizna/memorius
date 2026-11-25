import { PlusIcon, XIcon } from "lucide-react"
import type { GitStatus } from "@/shared/api/generated/GitStatus"
import { formatClock } from "@/shared/lib/local-date"
import { Button } from "@/shared/ui/button"
import { Input } from "@/shared/ui/input"
import { SettingsSection } from "@/shared/ui/settings-section"

export type GitSectionProps = {
  status: GitStatus | undefined
  saving: boolean
  error: string | undefined
  onRootsChange: (roots: string[]) => void
}

/** Where Memorius looks for repositories, and what it found. */
export function GitSection({
  status,
  saving,
  error,
  onRootsChange,
}: GitSectionProps) {
  const scan = status?.scan
  const roots = status?.settings.roots ?? []
  const add = (root: string) => {
    const folder = root.trim()
    if (folder && !roots.includes(folder)) onRootsChange([...roots, folder])
  }
  return (
    <SettingsSection
      id="git-heading"
      title="Git"
      description="Your own commits on this computer show in the Day view's Commits lane. Memorius reads the repositories' reflogs in the folders below and in every folder you or your agents worked in; on hover it reads one commit's message and changed file names, never file contents."
    >
      {status && (
        <div className="glass flex flex-col gap-3 rounded-2xl px-4 py-3.5 text-[13px]">
          <div className="flex flex-col gap-1.5 text-t4">
            <span>Repository folders</span>
            {roots.length > 0 ? (
              <ul className="m-0 flex list-none flex-col gap-1 p-0">
                {roots.map((root) => (
                  <li key={root} className="flex items-center gap-2">
                    <span className="font-mono text-t1">{root}</span>
                    <Button
                      size="icon-xs"
                      variant="ghost"
                      aria-label={`Remove ${root}`}
                      onClick={() =>
                        onRootsChange(roots.filter((other) => other !== root))
                      }
                    >
                      <XIcon aria-hidden="true" />
                    </Button>
                  </li>
                ))}
              </ul>
            ) : (
              <span className="text-t3">
                None: only folders you worked in are read.
              </span>
            )}
            <form
              className="flex max-w-96 gap-2"
              onSubmit={(event) => {
                event.preventDefault()
                const form = event.currentTarget
                const root = new FormData(form).get("root")
                if (typeof root === "string") add(root)
                form.reset()
              }}
            >
              <Input
                name="root"
                aria-label="Folder to add"
                className="font-mono"
                placeholder="~/code"
              />
              <Button type="submit" size="sm" variant="outline">
                Add
              </Button>
            </form>
            <span className="text-xs">Searched three levels deep.</span>
          </div>
          {scan && scan.suggestions.length > 0 && (
            <div className="flex flex-wrap items-center gap-2 text-t4">
              <span>Repositories found in</span>
              {scan.suggestions.map((folder) => (
                <Button
                  key={folder}
                  size="xs"
                  variant="outline"
                  onClick={() => add(folder)}
                >
                  <PlusIcon aria-hidden="true" />
                  <span className="font-mono">{folder}</span>
                </Button>
              ))}
            </div>
          )}
          <p className={`m-0 ${error ? "text-critical" : "text-t2"}`}>
            {error ??
              (saving
                ? "Reading commits…"
                : scan
                  ? `${scan.repositories} repositories · ${scan.commits} commits · read ${formatClock(scan.atMs)}`
                  : "Not read yet")}
          </p>
          {scan && (
            <p className="m-0 text-xs text-t4">
              Commits by{" "}
              {scan.emails.length > 0
                ? scan.emails.join(", ")
                : "nobody: set user.email in git config"}
              , from git config.
            </p>
          )}
        </div>
      )}
    </SettingsSection>
  )
}
