import type { DayCommit } from "@/shared/api/generated/DayCommit"

const ISSUE_KEY = /\b[A-Z][A-Z0-9]+-\d+\b/

/** The issue a commit names in its subject or branch. */
export function commitIssue(commit: DayCommit) {
  return (
    commit.subject.match(ISSUE_KEY)?.[0] ??
    commit.branch?.toUpperCase().match(ISSUE_KEY)?.[0] ??
    null
  )
}
