import { useQuery } from "@tanstack/react-query"
import {
  commitDetailsQuery,
  openLink,
  trackerStatusQuery,
} from "@/shared/api/memorius-commands"
import { Skeleton } from "@/shared/ui/skeleton"

const linkStyle =
  "rounded text-xs text-t2 underline-offset-2 outline-none hover:text-foreground hover:underline focus-visible:ring-2 focus-visible:ring-ring"

/** A commit's message, changed files and links, read from git while hovered. */
export function CommitDetailsFromGit({
  hash,
  issue,
}: {
  hash: string
  issue: string | null
}) {
  const details = useQuery(commitDetailsQuery(hash))
  const tracker = useQuery(trackerStatusQuery()).data?.url
  if (details.isPending) return <Skeleton className="h-3.5 w-56" />
  if (details.isError) {
    return <span className="text-xs text-t4">Details no longer available</span>
  }
  const { body, files, insertions, deletions, topFiles, webUrl } = details.data
  return (
    <>
      {body && (
        <span className="line-clamp-4 text-xs leading-normal whitespace-pre-line text-t2">
          {body}
        </span>
      )}
      <span className="flex flex-col gap-0.5 border-t border-border pt-2 text-xs text-t3">
        <span>
          {files} file{files === 1 ? "" : "s"} ·{" "}
          <span className="text-operational">+{insertions}</span>{" "}
          <span className="text-critical">−{deletions}</span>
        </span>
        {topFiles.map((file) => (
          <span key={file} className="truncate font-mono text-[11px] text-t4">
            {file}
          </span>
        ))}
      </span>
      {(webUrl || (issue && tracker)) && (
        <span className="flex gap-3">
          {issue && tracker && (
            <button
              type="button"
              className={linkStyle}
              onClick={() => void openLink(`${tracker}/issue/${issue}`)}
            >
              {issue} in YouTrack
            </button>
          )}
          {webUrl && (
            <button
              type="button"
              className={linkStyle}
              onClick={() => void openLink(webUrl)}
            >
              Open on GitHub
            </button>
          )}
        </span>
      )}
    </>
  )
}
