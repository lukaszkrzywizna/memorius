import { fireEvent, render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { SettingsPage } from "@/pages/settings/ui/settings-page"

const tracker = {
  status: undefined,
  busy: false,
  connectedAs: undefined,
  error: undefined,
  onConnect: vi.fn(),
  onSync: vi.fn(),
  onDisconnect: vi.fn(),
}

const reminder = {
  settings: undefined,
  onChange: vi.fn(),
  onTest: vi.fn(),
  testing: false,
  error: undefined,
}

const git = {
  status: undefined,
  saving: false,
  error: undefined,
  onRootsChange: vi.fn(),
}

const updates = {
  status: undefined,
  checking: false,
  installing: false,
  error: undefined,
  onCheck: vi.fn(),
  onInstall: vi.fn(),
}

const handlers = {
  onOpenSetup: vi.fn(),
  advanced: { rules: undefined, onChange: vi.fn() },
  git,
  updates,
  reminder,
  tracker,
  onRequestTitleAccess: vi.fn(),
  onPause: vi.fn(),
  onResume: vi.fn(),
  agent: {
    settings: undefined,
    check: undefined,
    prompt: undefined,
    onChange: vi.fn(),
    commandLine: { note: undefined, onInstall: vi.fn(), onAdd: vi.fn() },
  },
}

describe("SettingsPage", () => {
  it("installs the command and adds Memorius to an agent", () => {
    const onInstall = vi.fn()
    const onAdd = vi.fn()
    render(
      <SettingsPage
        {...handlers}
        status={undefined}
        agent={{
          ...handlers.agent,
          settings: {
            agent: "claudeCode",
            customCommand: "",
            daily: false,
            dailyTime: "08:00",
            defaultRange: "sinceLastRun",
            lookupTools: "",
            instructions: "",
            claudeHome: "",
            codexHome: "",
          },
          commandLine: { note: "Installed in ~/.local/bin.", onInstall, onAdd },
        }}
      />,
    )

    fireEvent.click(screen.getByRole("button", { name: "Install command" }))
    fireEvent.click(screen.getByRole("button", { name: "Add to Codex" }))
    expect(onInstall).toHaveBeenCalledOnce()
    expect(onAdd).toHaveBeenCalledWith("codex")
    expect(screen.getByRole("status")).toHaveTextContent("Installed in")
  })

  it("shows the installed version even when the check finds no release", () => {
    render(
      <SettingsPage
        {...handlers}
        status={undefined}
        updates={{
          ...updates,
          status: {
            current: "0.1.1",
            available: null,
            problem: "No published release yet",
          },
        }}
      />,
    )

    expect(screen.getByText("Memorius 0.1.1")).toBeInTheDocument()
    expect(screen.getByText("No published release yet.")).toBeInTheDocument()
  })

  it("saves a changed time rule and resets them all", () => {
    const onChange = vi.fn()
    const rules = {
      awayAfterMinutes: 5,
      breakMinutes: 15,
      agentAfterPromptMinutes: 30,
      weekStartsOn: "monday" as const,
    }
    render(
      <SettingsPage
        {...handlers}
        status={undefined}
        advanced={{ rules: { ...rules, breakMinutes: 20 }, onChange }}
      />,
    )

    const breaks = screen.getByLabelText("Breaks counted as work")
    fireEvent.change(breaks, { target: { value: "10" } })
    fireEvent.blur(breaks)
    expect(onChange).toHaveBeenLastCalledWith({ ...rules, breakMinutes: 10 })
    fireEvent.click(screen.getByRole("button", { name: "Reset to defaults" }))
    expect(onChange).toHaveBeenLastCalledWith(rules)
  })

  it("adds a suggested repository folder and removes one", () => {
    const onRootsChange = vi.fn()
    render(
      <SettingsPage
        {...handlers}
        status={undefined}
        git={{
          ...git,
          onRootsChange,
          status: {
            settings: { roots: ["~/code"] },
            scan: {
              atMs: 0,
              repositories: 5,
              commits: 9,
              emails: ["you@example.com"],
              suggestions: ["~/work"],
            },
          },
        }}
      />,
    )

    fireEvent.click(screen.getByRole("button", { name: "~/work" }))
    expect(onRootsChange).toHaveBeenLastCalledWith(["~/code", "~/work"])
    fireEvent.click(screen.getByRole("button", { name: "Remove ~/code" }))
    expect(onRootsChange).toHaveBeenLastCalledWith([])
  })

  it("offers a newer release and installs it", () => {
    const onInstall = vi.fn()
    render(
      <SettingsPage
        {...handlers}
        status={undefined}
        updates={{
          ...updates,
          status: { current: "0.1.0", available: "0.2.0", problem: null },
          onInstall,
        }}
      />,
    )

    expect(screen.getByText("Version 0.2.0 is available.")).toBeInTheDocument()
    fireEvent.click(screen.getByRole("button", { name: "Install and restart" }))
    expect(onInstall).toHaveBeenCalledOnce()
  })

  it("asks for Accessibility only when titles are hidden", () => {
    render(
      <SettingsPage
        {...handlers}
        status={{
          state: { kind: "recording", current: null },
          titleAccess: "denied",
          titlesLostMs: null,
        }}
      />,
    )

    fireEvent.click(screen.getByRole("button", { name: "Allow window titles" }))

    expect(handlers.onRequestTitleAccess).toHaveBeenCalledOnce()
  })

  it("pauses for the chosen time and offers resume while paused", () => {
    const { rerender } = render(
      <SettingsPage
        {...handlers}
        status={{
          state: { kind: "recording", current: null },
          titleAccess: "granted",
          titlesLostMs: null,
        }}
      />,
    )
    fireEvent.click(screen.getByRole("button", { name: "Pause for 1 hour" }))
    expect(handlers.onPause).toHaveBeenCalledWith(60)

    rerender(
      <SettingsPage
        {...handlers}
        status={{
          state: { kind: "paused", untilMs: 0 },
          titleAccess: "granted",
          titlesLostMs: null,
        }}
      />,
    )
    fireEvent.click(screen.getByRole("button", { name: "Resume now" }))
    expect(handlers.onResume).toHaveBeenCalledOnce()
  })

  it("saves the chosen agent and shows where its CLI was found", () => {
    const onChange = vi.fn()
    const settings = {
      agent: "claudeCode" as const,
      customCommand: "",
      daily: false,
      dailyTime: "08:00",
      defaultRange: "sinceLastRun" as const,
      lookupTools: "",
      claudeHome: "",
      codexHome: "",
      instructions: "",
    }
    render(
      <SettingsPage
        {...handlers}
        status={undefined}
        agent={{
          settings,
          check: {
            program: "claude",
            programPath: "/opt/bin/claude",
            memoriusPath: null,
          },
          prompt: undefined,
          onChange,
          commandLine: handlers.agent.commandLine,
        }}
      />,
    )

    fireEvent.change(screen.getByRole("combobox", { name: "Agent" }), {
      target: { value: "codex" },
    })

    expect(onChange).toHaveBeenCalledWith({ ...settings, agent: "codex" })
    expect(screen.getByText("/opt/bin/claude")).toBeInTheDocument()
    expect(
      screen.getByText("Not found in your shell's PATH"),
    ).toBeInTheDocument()
  })

  it("connects YouTrack with the address and a new token", () => {
    const onConnect = vi.fn()
    render(
      <SettingsPage
        {...handlers}
        status={undefined}
        tracker={{
          ...tracker,
          status: { url: "", hasToken: false, sync: { kind: "never" } },
          onConnect,
        }}
      />,
    )

    fireEvent.change(screen.getByLabelText("Address"), {
      target: { value: "https://example.youtrack.cloud" },
    })
    fireEvent.change(screen.getByLabelText(/Permanent token/), {
      target: { value: "perm:abc" },
    })
    fireEvent.click(screen.getByRole("button", { name: "Connect" }))

    expect(onConnect).toHaveBeenCalledWith(
      "https://example.youtrack.cloud",
      "perm:abc",
    )
    expect(screen.getByText("Not connected")).toBeInTheDocument()
  })

  it("turns the daily reminder on and sends a test", () => {
    const onChange = vi.fn()
    const onTest = vi.fn()
    render(
      <SettingsPage
        {...handlers}
        status={undefined}
        reminder={{
          ...reminder,
          settings: { enabled: false, time: "16:00" },
          onChange,
          onTest,
        }}
      />,
    )

    fireEvent.click(
      screen.getByRole("checkbox", { name: "Remind me every day at" }),
    )
    fireEvent.click(screen.getByRole("button", { name: "Send a test" }))

    expect(onChange).toHaveBeenCalledWith({ enabled: true, time: "16:00" })
    expect(onTest).toHaveBeenCalledOnce()
  })
})
