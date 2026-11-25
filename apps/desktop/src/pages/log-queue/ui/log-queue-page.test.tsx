import { fireEvent, render, screen, within } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { LogQueuePage } from "@/pages/log-queue/ui/log-queue-page"
import {
  doneRun,
  runningRun,
  sampleAgentSettings,
} from "@/pages/log-queue/ui/sample-agent.fixture"
import { sampleQueue } from "@/pages/log-queue/ui/sample-queue.fixture"

const agentHandlers = () => ({ onPrepare: vi.fn(), onStop: vi.fn() })

const youtrack = (onLog = vi.fn()) => ({
  onLog,
  connected: true,
  pending: false,
  result: undefined,
  error: undefined,
})

const setup = (
  failed: { id: number; issue: string; message: string }[] = [],
  runs = [doneRun],
) => {
  const handlers = {
    onApprove: vi.fn(),
    onReject: vi.fn(),
    onUndo: vi.fn(),
    onApproveMany: vi.fn(),
  }
  const agent = agentHandlers()
  const onLog = vi.fn()
  render(
    <LogQueuePage
      items={sampleQueue}
      {...handlers}
      bulk={{ pending: false, failed }}
      workTypes={["Development", "Investigation"]}
      daySummaries={[]}
      agent={{ settings: sampleAgentSettings, runs, ...agent }}
      youtrack={youtrack(onLog)}
    />,
  )
  return { ...handlers, ...agent, onLog }
}

const table = () => screen.getByRole("table", { name: "Proposals" })

describe("LogQueuePage", () => {
  it("lists proposals as rows under group dividers", () => {
    setup()

    const cells = within(table())
      .getAllByRole("cell")
      .map((cell) => cell.textContent)
    expect(cells).toContain("To approve")
    expect(cells).toContain("Approved · not logged yet")
    expect(screen.getByText("Logged · 2-48211")).toBeInTheDocument()
  })

  it("approves from the row, or with changes from the details", () => {
    const handlers = setup()

    fireEvent.click(screen.getByRole("button", { name: /^Approve SEL-54/ }))
    expect(handlers.onApprove).toHaveBeenCalledWith(1, null)

    fireEvent.click(screen.getByRole("button", { name: /^Details of SEL-54/ }))
    const details = screen.getByRole("complementary", {
      name: "Proposal details",
    })
    fireEvent.click(within(details).getByRole("button", { name: "Edit" }))
    fireEvent.change(within(details).getByLabelText(/Minutes/), {
      target: { value: "90" },
    })
    expect(
      within(details).getByRole("button", { name: "Approve with changes" }),
    ).toBeDisabled()
    fireEvent.change(within(details).getByLabelText(/Minutes/), {
      target: { value: "60" },
    })
    // Only YouTrack's own work types are offered.
    fireEvent.change(within(details).getByLabelText("Work type"), {
      target: { value: "Investigation" },
    })
    fireEvent.click(
      within(details).getByRole("button", { name: "Approve with changes" }),
    )

    expect(handlers.onApprove).toHaveBeenLastCalledWith(1, {
      issue: "SEL-54",
      minutes: 60,
      workType: "Investigation",
      text: "Moved the customer history dashboard to the app stack and checked the new Container App revision.",
    })
  })

  it("rejects, undoes and shows why logging failed", () => {
    const handlers = setup()

    fireEvent.click(screen.getByRole("button", { name: /^Reject IS-61/ }))
    fireEvent.click(screen.getByRole("button", { name: /^Undo INT-177/ }))
    fireEvent.click(screen.getByRole("button", { name: /^Details of INT-177/ }))

    expect(handlers.onReject).toHaveBeenCalledWith(2)
    expect(handlers.onUndo).toHaveBeenCalledWith(3)
    expect(screen.getByRole("alert")).toHaveTextContent("403")
  })

  it("approves the selected waiting rows at once", () => {
    const handlers = setup()

    fireEvent.click(
      screen.getByRole("checkbox", { name: "Select all waiting" }),
    )
    fireEvent.click(screen.getByRole("button", { name: "Approve 2 selected" }))

    expect(handlers.onApproveMany).toHaveBeenCalledWith([1, 2])
  })

  it("retries only the proposals a bulk approval missed", () => {
    const handlers = setup([{ id: 2, issue: "IS-61", message: "locked" }])

    expect(screen.getByRole("alert")).toHaveTextContent(
      "Could not approve IS-61: locked",
    )
    fireEvent.click(screen.getByRole("button", { name: "Retry these" }))
    expect(handlers.onApproveMany).toHaveBeenCalledWith([2])
  })

  it("starts the agent for a range; Memorius logs approved ones apart", () => {
    const { onPrepare, onLog } = setup()
    const bar = screen.getByRole("region", { name: "Agent" })

    fireEvent.change(within(bar).getByLabelText("Range"), {
      target: { value: "week" },
    })
    fireEvent.click(
      within(bar).getByRole("button", { name: "Prepare proposals" }),
    )
    expect(
      within(bar).queryByRole("button", { name: /YouTrack/ }),
    ).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole("button", { name: "Log 1 to YouTrack" }))

    expect(onPrepare).toHaveBeenCalledWith("week")
    expect(onLog).toHaveBeenCalledOnce()
  })

  it("logs one approved row from its own button", () => {
    const { onLog } = setup()

    fireEvent.click(
      within(table()).getByRole("button", {
        name: /^Log INT-177, .+ to YouTrack$/,
      }),
    )

    expect(onLog).toHaveBeenCalledWith([3])
  })

  it("folds a long agent summary behind Show more", () => {
    const summary = "I made 5 proposals. ".repeat(30)
    setup([], [{ ...doneRun, state: { kind: "done", finishedMs: 0, summary } }])
    const bar = screen.getByRole("region", { name: "Agent" })

    expect(within(bar).getByText("Done")).toBeInTheDocument()
    expect(within(bar).getByText(summary.trim())).toHaveClass("line-clamp-4")
    fireEvent.click(within(bar).getByRole("button", { name: "Show more" }))
    expect(within(bar).getByText(summary.trim())).not.toHaveClass(
      "line-clamp-4",
    )
  })

  it("sends to Settings to connect YouTrack before it can log", () => {
    const onLog = vi.fn()
    render(
      <LogQueuePage
        items={sampleQueue}
        onApprove={vi.fn()}
        onReject={vi.fn()}
        onUndo={vi.fn()}
        onApproveMany={vi.fn()}
        bulk={{ pending: false, failed: [] }}
        workTypes={[]}
        daySummaries={[]}
        agent={{ ...agentHandlers(), settings: sampleAgentSettings, runs: [] }}
        youtrack={{ ...youtrack(onLog), connected: false }}
      />,
    )

    fireEvent.click(
      screen.getByRole("button", { name: "Connect YouTrack to log" }),
    )
    expect(onLog).toHaveBeenCalledOnce()
  })

  it("shows a running run with a Stop button only", () => {
    const { onStop } = setup([], [runningRun])
    const bar = screen.getByRole("region", { name: "Agent" })

    expect(within(bar).getByText("Preparing proposals")).toBeInTheDocument()
    expect(
      within(bar).queryByRole("button", { name: "Prepare proposals" }),
    ).not.toBeInTheDocument()
    fireEvent.click(within(bar).getByRole("button", { name: "Stop" }))
    expect(onStop).toHaveBeenCalledOnce()
  })

  it("offers the agent in one card when the queue is empty", () => {
    const onPrepare = vi.fn()
    render(
      <LogQueuePage
        items={[]}
        onApprove={vi.fn()}
        onReject={vi.fn()}
        onUndo={vi.fn()}
        onApproveMany={vi.fn()}
        bulk={{ pending: false, failed: [] }}
        workTypes={[]}
        daySummaries={[]}
        agent={{
          settings: sampleAgentSettings,
          runs: [],
          onPrepare,
          onStop: vi.fn(),
        }}
        youtrack={youtrack()}
      />,
    )
    const card = screen.getByRole("region", { name: "Agent" })

    expect(within(card).getByText("No proposals yet")).toBeInTheDocument()
    expect(
      screen.queryByRole("group", { name: "Show" }),
    ).not.toBeInTheDocument()
    fireEvent.click(
      within(card).getByRole("button", { name: "Prepare proposals" }),
    )
    expect(onPrepare).toHaveBeenCalledWith("sinceLastRun")
  })

  it("reports what Memorius logged and which worklogs failed", () => {
    render(
      <LogQueuePage
        items={sampleQueue}
        onApprove={vi.fn()}
        onReject={vi.fn()}
        onUndo={vi.fn()}
        onApproveMany={vi.fn()}
        bulk={{ pending: false, failed: [] }}
        workTypes={[]}
        daySummaries={[]}
        agent={{
          ...agentHandlers(),
          settings: sampleAgentSettings,
          runs: [],
        }}
        youtrack={{
          onLog: vi.fn(),
          connected: true,
          pending: false,
          result: {
            logged: 24,
            failed: [
              {
                issue: "SEL-3",
                date: "2026-10-12",
                error: "YouTrack has no work type Meeting",
              },
            ],
          },
          error: undefined,
        }}
      />,
    )

    expect(screen.getByRole("status")).toHaveTextContent(
      "Logged 24 in YouTrack. SEL-3 2026-10-12: YouTrack has no work type Meeting",
    )
  })

  it("groups each day under a row that sums it, approves it and folds it", () => {
    const { onApproveMany } = setup()
    const friday = screen.getByRole("row", { name: "Fri 9 Oct, 1h 19m to log" })

    fireEvent.click(within(friday).getByRole("button", { name: "Approve day" }))
    expect(onApproveMany).toHaveBeenCalledWith([1])

    fireEvent.click(within(friday).getByRole("button", { name: "Fri 9 Oct" }))
    expect(
      screen.queryByRole("button", { name: /^Approve SEL-54/ }),
    ).not.toBeInTheDocument()
  })
})
