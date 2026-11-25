import { describe, expect, it } from "vitest"
import { commitIssue } from "@/pages/day/lib/commit-issue"

const commit = (subject: string, branch: string | null) => ({
  hash: "a",
  atMs: 0,
  subject,
  repo: "Shop",
  branch,
})

describe("commitIssue", () => {
  it("takes the issue from the subject, then from the branch", () => {
    expect(commitIssue(commit("[SEL-74] Merge main", "main"))).toBe("SEL-74")
    expect(commitIssue(commit("Merge main", "claude/int-290-push"))).toBe(
      "INT-290",
    )
    expect(commitIssue(commit("Merge main", null))).toBeNull()
  })
})
