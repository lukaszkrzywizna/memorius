import type { Evidence } from "@/shared/api/generated/Evidence"
import type { RuleKind } from "@/shared/api/generated/RuleKind"
import type { WorkSpan } from "@/shared/api/generated/WorkSpan"

/** Short text for a work span: its first session or window title, or commit. */
export function workLabel(span: WorkSpan) {
  const evidence = span.evidence[0]
  if (!evidence) return ""
  switch (evidence.kind) {
    case "windowTitle":
      return evidence.title
    case "url":
      return evidence.url
    case "session":
      return evidence.title ?? evidence.branch ?? ""
    case "commit":
      return evidence.subject
    case "rule":
      return ruleText(evidence.rule)
  }
}

/** Where the first evidence of a work span came from. */
export function workSource(span: WorkSpan) {
  return evidenceName(span.evidence[0])
}

function evidenceName(evidence: Evidence | undefined) {
  if (!evidence) return ""
  switch (evidence.kind) {
    case "windowTitle":
      return `${evidence.app} window`
    case "url":
      return `${evidence.app} address`
    case "session":
      return evidence.source === "claudeCode"
        ? "Claude Code session"
        : "Codex session"
    case "commit":
      return "commit"
    case "rule":
      return "your rule"
  }
}

/** What a rule matched, in a few words. */
export function ruleText(rule: RuleKind) {
  switch (rule.kind) {
    case "privateApp":
      return "private app"
    case "appIssue":
      return `app ${rule.appId}`
    case "hostIssue":
      return `website ${rule.host}`
    case "titleIssue":
      return `title contains “${rule.text}”`
  }
}
