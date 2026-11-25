const ISSUE_KEY = /^[A-Za-z][A-Za-z0-9]{1,9}-[1-9]\d*$/

/** A YouTrack issue key such as `INT-290`, in any case. */
export function isIssueKey(text: string) {
  return ISSUE_KEY.test(text.trim())
}
