import { describe, expect, it } from "vitest"
import { isIssueKey } from "@/entities/issue"
import { appNames, issueRule, ruleCells } from "@/pages/rules/lib/rule-text"

describe("rule text", () => {
  it("accepts issue keys in any case and rejects other text", () => {
    expect(["SEL-54", "int-290", " IS-61 "].every(isIssueKey)).toBe(true)
    expect(["SEL", "SEL-", "SEL-054", "UTF 8"].some(isIssueKey)).toBe(false)
  })

  it("shows rule cells with app names when known", () => {
    const name = appNames([
      { id: "com.microsoft.teams2", name: "Microsoft Teams" },
    ])

    expect(
      ruleCells(
        { kind: "appIssue", appId: "com.microsoft.teams2", issue: "SEL-21" },
        name,
      ),
    ).toEqual({ match: "App", pattern: "Microsoft Teams", issue: "SEL-21" })
    expect(
      ruleCells(
        { kind: "titleIssue", text: "Stand-up", issue: "SEL-21" },
        name,
      ),
    ).toEqual({ match: "Title contains", pattern: "Stand-up", issue: "SEL-21" })
  })

  it("builds the rule for each kind", () => {
    expect(issueRule("hostIssue", "example.com", "SEL-54")).toEqual({
      kind: "hostIssue",
      host: "example.com",
      issue: "SEL-54",
    })
  })
})
