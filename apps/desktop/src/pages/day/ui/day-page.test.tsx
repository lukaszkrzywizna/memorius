import { fireEvent, render, screen, within } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { DayPage } from "@/pages/day/ui/day-page"
import { formatDuration } from "@/shared/lib/duration"
import {
  emptyDay,
  sampleCommits,
  sampleDate,
  sampleDay,
} from "@/pages/day/ui/sample-day.fixture"

const ready = { kind: "ready" as const, timeline: sampleDay }
/** Props every test shares: horizontal view, nothing highlighted. */
const extra = {
  view: "horizontal" as const,
  onViewChange: vi.fn(),
  unloggedMs: 172 * 60_000,
  nowMs: new Date(2026, 9, 11, 12).getTime(),
  highlight: null,
  onClearHighlight: vi.fn(),
  PromptText: () => "Run the order sync tests again.",
  CommitDetails: ({ issue }: { issue: string | null }) => `4 files · ${issue}`,
}

describe("DayPage", () => {
  it("shows the totals and one button per app block", () => {
    render(
      <DayPage
        date={sampleDate}
        today="2026-10-11"
        state={ready}
        onDateChange={vi.fn()}
        zoom={15}
        onZoomChange={vi.fn()}
        {...extra}
        onMarkPrivate={vi.fn()}
      />,
    )

    expect(screen.getByRole("heading", { level: 1 })).toHaveTextContent(
      "Friday",
    )
    expect(screen.getByLabelText("Day totals")).toHaveTextContent(
      `Your work${formatDuration(sampleDay.totals.split.workMs)}`,
    )
    expect(
      within(screen.getByLabelText("Apps")).getAllByRole("button"),
    ).toHaveLength(5)
    expect(
      within(screen.getByLabelText("Timeline")).getByText("Away"),
    ).toBeInTheDocument()
  })

  it("lists the windows of the selected block", () => {
    render(
      <DayPage
        date={sampleDate}
        today="2026-10-11"
        state={ready}
        onDateChange={vi.fn()}
        zoom={15}
        onZoomChange={vi.fn()}
        {...extra}
        onMarkPrivate={vi.fn()}
      />,
    )

    fireEvent.click(screen.getByRole("button", { name: /^OrderSync\.fs/ }))

    const inspector = screen.getByLabelText("Selected activity")
    expect(
      within(inspector).getByText("OrderSyncTests.fs — Shop"),
    ).toBeInTheDocument()
    expect(within(inspector).getByText("52m")).toBeInTheDocument()
  })

  it("shows one work context span per issue run", () => {
    render(
      <DayPage
        date={sampleDate}
        today="2026-10-11"
        state={ready}
        onDateChange={vi.fn()}
        zoom={15}
        onZoomChange={vi.fn()}
        {...extra}
        onMarkPrivate={vi.fn()}
      />,
    )

    const lane = screen.getByLabelText("Work context")
    expect(
      within(lane)
        .getAllByText(/^(SEL-54|INT-290|INT-177|IS-61)\??$/)
        .map((key) => key.textContent),
    ).toEqual(["SEL-54?", "INT-290", "INT-177", "IS-61?", "IS-61"])
  })

  it("shows the evidence of a work span in its own card", () => {
    render(
      <DayPage
        date={sampleDate}
        today="2026-10-11"
        state={ready}
        onDateChange={vi.fn()}
        zoom={15}
        onZoomChange={vi.fn()}
        {...extra}
        onMarkPrivate={vi.fn()}
      />,
    )

    fireEvent.click(
      within(screen.getByLabelText("Work context")).getByRole("button", {
        name: /^INT-290,/,
      }),
    )

    const card = screen.getByLabelText("Detected work")
    expect(
      within(card).getByText("[INT-290] Skip unchanged employees"),
    ).toBeInTheDocument()
    expect(screen.queryByLabelText("Selected activity")).not.toBeInTheDocument()
  })

  it("shows what YouTrack holds next to the work, with the gaps", () => {
    const onOpenIssue = vi.fn()
    render(
      <DayPage
        date={sampleDate}
        today="2026-10-11"
        state={ready}
        onDateChange={vi.fn()}
        zoom={15}
        onZoomChange={vi.fn()}
        {...extra}
        onMarkPrivate={vi.fn()}
        onOpenIssue={onOpenIssue}
      />,
    )

    const lane = screen.getByLabelText("YouTrack")
    expect(
      within(lane).getByRole("img", { name: "Not logged, 10:15–11:30" }),
    ).toBeInTheDocument()
    expect(
      within(lane).getByRole("button", { name: /^IS-61, 1h 15m, proposed/ }),
    ).toBeInTheDocument()
    fireEvent.click(
      within(lane).getByRole("button", {
        name: /^INT-177, 1h, logged · time estimated/,
      }),
    )

    const card = screen.getByLabelText("YouTrack worklog")
    expect(
      within(card).getByText("INT-177 · Sync migration batch 2"),
    ).toBeInTheDocument()
    fireEvent.click(
      within(card).getByRole("button", { name: "INT-177 in YouTrack" }),
    )
    expect(onOpenIssue).toHaveBeenCalledWith("INT-177")
  })

  it("shows agent sessions in their own lane with a card", () => {
    render(
      <DayPage
        date={sampleDate}
        today="2026-10-11"
        state={ready}
        onDateChange={vi.fn()}
        zoom={15}
        onZoomChange={vi.fn()}
        {...extra}
        onMarkPrivate={vi.fn()}
      />,
    )

    fireEvent.click(
      screen.getByRole("button", {
        name: /^Claude Code, Order Search readiness/,
      }),
    )

    const card = screen.getByLabelText("Agent session")
    expect(
      within(card).getByText("claude/is61-uuid-keys-pg"),
    ).toBeInTheDocument()
    expect(
      within(screen.getByLabelText("Day totals")).getByText("Agents counted"),
    ).toBeInTheDocument()
  })

  it("says when an agent session has no ticket", () => {
    const [first, ...rest] = sampleDay.agents
    const timeline = {
      ...sampleDay,
      agents: first ? [{ ...first, issue: null }, ...rest] : rest,
    }
    render(
      <DayPage
        date={sampleDate}
        today="2026-10-11"
        state={{ kind: "ready", timeline }}
        onDateChange={vi.fn()}
        zoom={15}
        onZoomChange={vi.fn()}
        {...extra}
        onMarkPrivate={vi.fn()}
      />,
    )

    expect(
      screen.getByRole("button", { name: /, no ticket found$/ }),
    ).toHaveAttribute("title", "No ticket found")
  })

  it("marks a whole session private from the work card", () => {
    const onMarkPrivate = vi.fn()
    render(
      <DayPage
        date={sampleDate}
        today="2026-10-11"
        state={ready}
        onDateChange={vi.fn()}
        zoom={15}
        onZoomChange={vi.fn()}
        {...extra}
        onMarkPrivate={onMarkPrivate}
      />,
    )

    fireEvent.click(screen.getByRole("button", { name: /^IS-61\?, / }))
    const card = screen.getByLabelText("Detected work")
    expect(
      within(card).getByText("45m counted while you were away"),
    ).toBeInTheDocument()
    fireEvent.click(within(card).getByRole("button", { name: "Mark private" }))

    expect(onMarkPrivate).toHaveBeenCalledWith({
      sessions: [{ source: "claudeCode", sessionId: "is61" }],
      private: true,
    })
    expect(
      within(screen.getByLabelText("Day totals")).queryByText("To confirm"),
    ).not.toBeInTheDocument()
  })

  it("changes the zoom", () => {
    const onZoomChange = vi.fn()
    render(
      <DayPage
        date={sampleDate}
        today="2026-10-11"
        state={ready}
        onDateChange={vi.fn()}
        zoom={15}
        onZoomChange={onZoomChange}
        {...extra}
        onMarkPrivate={vi.fn()}
      />,
    )

    fireEvent.click(
      within(screen.getByRole("toolbar", { name: "Timeline view" })).getByRole(
        "button",
        { name: "Zoom 1h slots" },
      ),
    )

    expect(onZoomChange).toHaveBeenCalledWith(60)
  })

  it("moves between days and cannot go past today", () => {
    const onDateChange = vi.fn()
    render(
      <DayPage
        date="2026-10-11"
        today="2026-10-11"
        state={ready}
        onDateChange={onDateChange}
        zoom={15}
        onZoomChange={vi.fn()}
        {...extra}
        onMarkPrivate={vi.fn()}
      />,
    )

    fireEvent.click(screen.getByRole("button", { name: "Previous day" }))

    expect(onDateChange).toHaveBeenCalledWith("2026-10-10")
    expect(screen.getByRole("button", { name: "Next day" })).toBeDisabled()
  })

  it("says so when a day has no activity", () => {
    render(
      <DayPage
        date={sampleDate}
        today="2026-10-11"
        state={{ kind: "ready", timeline: emptyDay }}
        onDateChange={vi.fn()}
        zoom={15}
        onZoomChange={vi.fn()}
        {...extra}
        onMarkPrivate={vi.fn()}
      />,
    )

    expect(
      screen.getByText("No activity was recorded on this day."),
    ).toBeInTheDocument()
  })

  it("switches between the horizontal and the vertical timeline", () => {
    const onViewChange = vi.fn()
    render(
      <DayPage
        date={sampleDate}
        today="2026-10-11"
        state={ready}
        onDateChange={vi.fn()}
        zoom={15}
        onZoomChange={vi.fn()}
        onMarkPrivate={vi.fn()}
        {...extra}
        view="vertical"
        onViewChange={onViewChange}
      />,
    )

    expect(
      screen.getByRole("button", { name: "Vertical timeline" }),
    ).toHaveAttribute("aria-pressed", "true")
    fireEvent.click(screen.getByRole("button", { name: "Horizontal timeline" }))
    expect(onViewChange).toHaveBeenCalledWith("horizontal")
  })

  it("dims everything but the highlighted issue and can stop", () => {
    const onClearHighlight = vi.fn()
    render(
      <DayPage
        date={sampleDate}
        today="2026-10-11"
        state={ready}
        onDateChange={vi.fn()}
        zoom={15}
        onZoomChange={vi.fn()}
        onMarkPrivate={vi.fn()}
        {...extra}
        highlight="INT-290"
        onClearHighlight={onClearHighlight}
      />,
    )

    const lane = screen.getByLabelText("Work context")
    expect(
      within(lane).getByRole("button", { name: /^INT-290,/ }),
    ).not.toHaveClass("opacity-50")
    expect(within(lane).getByRole("button", { name: /^INT-177,/ })).toHaveClass(
      "opacity-50",
    )
    fireEvent.click(
      screen.getByRole("button", { name: "Stop showing INT-290" }),
    )
    expect(onClearHighlight).toHaveBeenCalledOnce()
  })

  it("closes the details card", () => {
    render(
      <DayPage
        date={sampleDate}
        today="2026-10-11"
        state={ready}
        onDateChange={vi.fn()}
        zoom={15}
        onZoomChange={vi.fn()}
        onMarkPrivate={vi.fn()}
        {...extra}
      />,
    )

    fireEvent.click(screen.getByRole("button", { name: /^OrderSync\.fs/ }))
    fireEvent.click(screen.getByRole("button", { name: "Close details" }))

    expect(screen.queryByLabelText("Selected activity")).not.toBeInTheDocument()
  })

  it("closes the details card on a click outside it, not on another block", () => {
    render(
      <DayPage
        date={sampleDate}
        today="2026-10-11"
        state={ready}
        onDateChange={vi.fn()}
        zoom={15}
        onZoomChange={vi.fn()}
        onMarkPrivate={vi.fn()}
        {...extra}
      />,
    )

    fireEvent.click(screen.getByRole("button", { name: /^OrderSync\.fs/ }))
    fireEvent.pointerDown(screen.getByLabelText("Selected activity"))
    expect(screen.getByLabelText("Selected activity")).toBeInTheDocument()
    const other = within(screen.getByLabelText("Work context")).getByRole(
      "button",
      { name: /^INT-290,/ },
    )
    fireEvent.pointerDown(other)
    fireEvent.click(other)
    expect(screen.getByLabelText("Detected work")).toBeInTheDocument()

    fireEvent.pointerDown(document.body)

    expect(screen.queryByLabelText("Detected work")).not.toBeInTheDocument()
  })

  it("shows a prompt's text on hover", () => {
    render(
      <DayPage
        date={sampleDate}
        today="2026-10-11"
        state={ready}
        onDateChange={vi.fn()}
        zoom={15}
        onZoomChange={vi.fn()}
        onMarkPrivate={vi.fn()}
        {...extra}
      />,
    )

    fireEvent.mouseEnter(
      within(screen.getByLabelText("Your prompts")).getAllByRole("button")[0]!,
    )

    expect(screen.getByRole("tooltip")).toHaveTextContent(
      "Run the order sync tests again.",
    )
  })

  it("folds a lane to marks that still open their details", () => {
    localStorage.removeItem("memorius.day.collapsedLanes")
    render(
      <DayPage
        date={sampleDate}
        today="2026-10-11"
        state={ready}
        onDateChange={vi.fn()}
        zoom={15}
        onZoomChange={vi.fn()}
        onMarkPrivate={vi.fn()}
        {...extra}
      />,
    )

    fireEvent.click(screen.getByRole("button", { name: "Fold Apps" }))
    expect(
      screen.queryByRole("button", { name: /^OrderSync\.fs/ }),
    ).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole("button", { name: /^Rider · OrderSync/ }))

    expect(screen.getByLabelText("Selected activity")).toBeInTheDocument()
    expect(localStorage.getItem("memorius.day.collapsedLanes")).toBe('["apps"]')
    fireEvent.click(screen.getByRole("button", { name: "Unfold Apps" }))
    localStorage.removeItem("memorius.day.collapsedLanes")
  })

  it("shows the Commits lane only when there are commits", () => {
    const props = {
      date: sampleDate,
      today: "2026-10-11",
      state: ready,
      onDateChange: vi.fn(),
      zoom: 15 as const,
      onZoomChange: vi.fn(),
      onMarkPrivate: vi.fn(),
      ...extra,
    }
    const { rerender } = render(<DayPage {...props} />)
    expect(screen.queryByLabelText("Commits")).not.toBeInTheDocument()

    rerender(
      <DayPage
        {...props}
        state={{
          kind: "ready",
          timeline: { ...sampleDay, commits: sampleCommits },
        }}
      />,
    )

    expect(
      within(screen.getByLabelText("Commits")).getByRole("button", {
        name: /^13:47 · \[IS-61\] UUID keys on PostgreSQL · Shop/,
      }),
    ).toBeInTheDocument()
  })

  it("opens a commit's card with where, when and its details on hover", () => {
    render(
      <DayPage
        date={sampleDate}
        today="2026-10-11"
        state={{
          kind: "ready",
          timeline: { ...sampleDay, commits: sampleCommits },
        }}
        onDateChange={vi.fn()}
        zoom={15}
        onZoomChange={vi.fn()}
        onMarkPrivate={vi.fn()}
        {...extra}
      />,
    )

    fireEvent.focus(
      screen.getByRole("button", { name: /^13:47 · \[IS-61\] UUID keys/ }),
    )

    const card = screen.getByRole("dialog", { name: /^Commit / })
    expect(
      within(card).getByText("claude/is61-uuid-keys-pg"),
    ).toBeInTheDocument()
    expect(
      within(card).getByText(/^During Claude Code: Order Search/),
    ).toBeInTheDocument()
    expect(within(card).getByText("4 files · IS-61")).toBeInTheDocument()
  })
})
