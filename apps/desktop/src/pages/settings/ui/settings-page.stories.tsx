import type { Meta, StoryObj } from "@storybook/react-vite"
import { fn } from "storybook/test"
import { SettingsPage } from "@/pages/settings/ui/settings-page"

const meta = {
  title: "Apps/Desktop/Pages/Settings",
  tags: ["autodocs"],
  component: SettingsPage,
  parameters: { layout: "fullscreen" },
  args: {
    status: {
      state: { kind: "recording", current: null },
      titleAccess: "granted",
      titlesLostMs: null,
    },
    onRequestTitleAccess: fn(),
    onOpenSetup: fn(),
    advanced: {
      rules: {
        awayAfterMinutes: 5,
        breakMinutes: 15,
        agentAfterPromptMinutes: 30,
        weekStartsOn: "monday" as const,
      },
      onChange: fn(),
    },
    onPause: fn(),
    onResume: fn(),
    updates: {
      status: { current: "0.1.0", available: null, problem: null },
      checking: false,
      installing: false,
      error: undefined,
      onCheck: fn(),
      onInstall: fn(),
    },
    git: {
      status: {
        settings: { roots: ["~/code"] },
        scan: {
          atMs: new Date(2026, 9, 20, 12, 40).getTime(),
          repositories: 4,
          commits: 312,
          emails: ["you@example.com"],
          suggestions: ["~/work"],
        },
      },
      saving: false,
      error: undefined,
      onRootsChange: fn(),
    },
    reminder: {
      settings: { enabled: true, time: "16:00" },
      onChange: fn(),
      onTest: fn(),
      testing: false,
      error: undefined,
    },
    tracker: {
      status: {
        url: "https://example.youtrack.cloud",
        hasToken: true,
        sync: {
          kind: "synced",
          atMs: new Date(2026, 9, 16, 10, 42).getTime(),
          items: 132,
        },
      },
      busy: false,
      connectedAs: undefined,
      error: undefined,
      onConnect: fn(),
      onSync: fn(),
      onDisconnect: fn(),
    },
    agent: {
      settings: {
        agent: "claudeCode",
        customCommand: "",
        daily: true,
        dailyTime: "08:00",
        defaultRange: "sinceLastRun",
        lookupTools: "youtrack/get_issue,youtrack/search_issues",
        claudeHome: "",
        codexHome: "",
        instructions: "Round to 15 minutes.",
      },
      check: {
        program: "claude",
        programPath: "/Users/me/.local/bin/claude",
        memoriusPath: "/Users/me/.cargo/bin/memorius",
      },
      prompt:
        "You prepare the user's worklogs in Memorius from their recorded activity from 2026-10-14 to 2026-10-16 (local dates).\n…\n\nThe user's instructions:\nRound to 15 minutes.",
      onChange: fn(),
      commandLine: { note: undefined, onInstall: fn(), onAdd: fn() },
    },
  },
} satisfies Meta<typeof SettingsPage>

export default meta

type Story = StoryObj<typeof meta>

export const Recording: Story = {}

export const TitlesNotAllowed: Story = {
  tags: ["visual-smoke"],
  args: {
    status: {
      state: { kind: "recording", current: null },
      titleAccess: "denied",
      titlesLostMs: null,
    },
  },
}

export const NoPublishedRelease: Story = {
  args: {
    updates: {
      status: {
        current: "0.1.1",
        available: null,
        problem: "No published release yet",
      },
      checking: false,
      installing: false,
      error: undefined,
      onCheck: fn(),
      onInstall: fn(),
    },
  },
}

export const UpdateAvailable: Story = {
  args: {
    updates: {
      status: { current: "0.1.0", available: "0.2.0", problem: null },
      checking: false,
      installing: false,
      error: undefined,
      onCheck: fn(),
      onInstall: fn(),
    },
  },
}

export const Paused: Story = {
  args: {
    status: {
      state: {
        kind: "paused",
        untilMs: new Date(2026, 9, 9, 14, 30).getTime(),
      },
      titleAccess: "granted",
      titlesLostMs: null,
    },
  },
}
