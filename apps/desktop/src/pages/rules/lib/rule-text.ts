import type { AppRef } from "@/shared/api/generated/AppRef"
import type { RuleKind } from "@/shared/api/generated/RuleKind"

export type IssueRuleKind = "appIssue" | "hostIssue" | "titleIssue"

export function issueRule(
  kind: IssueRuleKind,
  pattern: string,
  issue: string,
): RuleKind {
  switch (kind) {
    case "appIssue":
      return { kind, appId: pattern, issue }
    case "hostIssue":
      return { kind, host: pattern, issue }
    case "titleIssue":
      return { kind, text: pattern, issue }
  }
}

/** Names for app ids from recent use; unknown ids show as they are. */
export function appNames(apps: readonly AppRef[]) {
  const names = new Map(apps.map((app) => [app.id, app.name]))
  return (appId: string) => names.get(appId) ?? appId
}

/** An issue rule as table cells: what kind of match, the pattern, the issue. */
export function ruleCells(
  rule: Exclude<RuleKind, { kind: "privateApp" }>,
  appName: (appId: string) => string,
) {
  switch (rule.kind) {
    case "appIssue":
      return { match: "App", pattern: appName(rule.appId), issue: rule.issue }
    case "hostIssue":
      return { match: "Website", pattern: rule.host, issue: rule.issue }
    case "titleIssue":
      return { match: "Title contains", pattern: rule.text, issue: rule.issue }
  }
}
