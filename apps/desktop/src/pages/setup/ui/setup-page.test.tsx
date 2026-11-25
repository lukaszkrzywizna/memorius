import { fireEvent, render, screen, within } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import type { SetupStep } from "@/pages/setup/lib/setup-steps"
import { SetupPage } from "@/pages/setup/ui/setup-page"

const steps = (titles: boolean): SetupStep[] => [
  { id: "notifications", required: false, done: true },
  { id: "titles", required: true, done: titles },
  { id: "login", required: false, done: false },
  { id: "repositories", required: false, done: true },
  { id: "agent", required: false, done: false },
  { id: "commandLine", required: false, done: false },
  { id: "tracker", required: false, done: false },
]

const actions = () => ({
  notifications: [],
  titles: [{ label: "Open System Settings", onClick: vi.fn() }],
  login: [{ label: "Turn on", onClick: vi.fn() }],
  repositories: [],
  agent: [{ label: "Sign in", onClick: vi.fn() }],
  commandLine: [],
  tracker: [{ label: "Connect", onClick: vi.fn() }],
})

describe("SetupPage", () => {
  it("starts recording only once the required step is done", () => {
    const onFinish = vi.fn()
    const props = {
      notes: {},
      waiting: null,
      folders: { suggestions: [], onAdd: vi.fn() },
      onFinish,
    }
    const { rerender } = render(
      <SetupPage {...props} steps={steps(false)} actions={actions()} />,
    )

    expect(
      screen.getByRole("button", { name: "Start recording" }),
    ).toBeDisabled()
    expect(
      screen.getByText("1 required left", { exact: false }),
    ).toBeInTheDocument()

    rerender(<SetupPage {...props} steps={steps(true)} actions={actions()} />)
    expect(
      screen.getByRole("listitem", { name: "Window titles, done" }),
    ).toBeInTheDocument()
    fireEvent.click(screen.getByRole("button", { name: "Start recording" }))
    expect(onFinish).toHaveBeenCalledOnce()
  })

  it("moves on to the next step when an optional one is skipped", () => {
    render(
      <SetupPage
        steps={steps(true)}
        actions={actions()}
        notes={{}}
        waiting={null}
        folders={{ suggestions: [], onAdd: vi.fn() }}
        onFinish={vi.fn()}
      />,
    )
    const login = screen.getByRole("listitem", {
      name: "Start at login, to do",
    })

    expect(login).toHaveAttribute("aria-current", "step")

    fireEvent.click(within(login).getByRole("button", { name: "Skip" }))

    expect(login).not.toHaveAttribute("aria-current")
    expect(
      screen.getByRole("listitem", { name: "Agent, to do" }),
    ).toHaveAttribute("aria-current", "step")
  })

  it("adds a repositories folder typed in or found", () => {
    const onAdd = vi.fn()
    const open = steps(true).map((step) =>
      step.id === "repositories" ? { ...step, done: false } : step,
    )
    render(
      <SetupPage
        steps={open}
        actions={actions()}
        notes={{}}
        waiting={null}
        folders={{ suggestions: ["~/work"], onAdd }}
        onFinish={vi.fn()}
      />,
    )

    fireEvent.click(screen.getByRole("button", { name: "~/work" }))
    fireEvent.change(screen.getByLabelText("Repositories folder"), {
      target: { value: " ~/code " },
    })
    fireEvent.click(screen.getByRole("button", { name: "Add" }))

    expect(onAdd.mock.calls).toEqual([["~/work"], ["~/code"]])
  })
})
