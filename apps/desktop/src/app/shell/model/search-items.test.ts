import { describe, expect, it } from "vitest"
import { dateGroups, resultGroups } from "@/app/shell/model/search-items"
import { sampleSearch } from "@/app/shell/ui/search.fixture"

describe("search groups", () => {
  it("groups results and keeps logged and proposed minutes apart", () => {
    const groups = resultGroups(sampleSearch).groups

    expect(groups.map((group) => group.title)).toEqual([
      "Issues",
      "Apps and windows",
      "Agent sessions",
    ])
    const issue = groups[0]?.items[0]
    expect(issue?.meta).toBe("40m logged · 5h proposed")
    expect(issue?.chips.map((chip) => chip.label)).toEqual([
      "Fri 9 Oct · 3h 25m",
      "Thu 8 Oct · 1h 35m",
      "Wed 7 Oct · 40m",
    ])
    expect(issue?.target).toEqual({
      page: "week",
      week: "2026-10-09",
      issue: "IS-61",
    })
    expect(issue?.chips[2]?.target).toEqual({
      page: "day",
      date: "2026-10-07",
      issue: "IS-61",
    })
  })

  it("opens a window's last day and a typed date", () => {
    const groups = [
      ...resultGroups(sampleSearch).groups,
      ...dateGroups("2026-09-28"),
    ]

    expect(groups[1]?.items[0]?.target).toEqual({
      page: "day",
      date: "2026-10-09",
      issue: "IS-61",
    })
    expect(groups.at(-1)?.items[0]?.target).toEqual({
      page: "day",
      date: "2026-09-28",
    })
  })
})
