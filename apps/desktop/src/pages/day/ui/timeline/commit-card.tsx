import { CopyIcon } from "lucide-react"
import { type ComponentType, useState } from "react"
import type { DayCommit } from "@/shared/api/generated/DayCommit"
import { commitIssue } from "@/pages/day/lib/commit-issue"
import { formatClock } from "@/shared/lib/local-date"

/** A commit's message, changes and links; the app reads them from git on hover. */
export type CommitDetailsView = ComponentType<{
  hash: string
  issue: string | null
}>

type CommitCardProps = {
  commit: DayCommit
  /** What ran at that moment: the agent session and the work span. */
  context: string | null
  CommitDetails: CommitDetailsView
  className: string
}

/** The hover card of a commit node: subject, where and when, and details. */
export function CommitCard({
  commit,
  context,
  CommitDetails,
  className,
}: CommitCardProps) {
  const [copied, setCopied] = useState(false)
  const short = commit.hash.slice(0, 7)
  return (
    <div
      role="dialog"
      aria-label={`Commit ${short}`}
      className={`frost absolute z-30 flex w-[340px] flex-col gap-2 rounded-2xl px-3.5 py-3 text-left ${className}`}
    >
      <span className="text-[13px] leading-normal text-t1">
        {commit.subject}
      </span>
      <span className="flex flex-wrap items-center gap-x-1.5 text-xs text-t4">
        <span>{formatClock(commit.atMs)}</span>·<span>{commit.repo}</span>
        {commit.branch && (
          <>
            ·<span className="max-w-40 truncate">{commit.branch}</span>
          </>
        )}
        ·
        <button
          type="button"
          title="Copy the full hash"
          onClick={() =>
            void navigator.clipboard
              .writeText(commit.hash)
              .then(() => setCopied(true))
              .catch(() => setCopied(false))
          }
          className="inline-flex items-center gap-1 rounded font-mono text-t3 outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
        >
          {copied ? "copied" : short}
          <CopyIcon aria-hidden="true" className="size-3" />
        </button>
      </span>
      {context && <span className="text-xs text-t3">{context}</span>}
      <CommitDetails hash={commit.hash} issue={commitIssue(commit)} />
    </div>
  )
}
