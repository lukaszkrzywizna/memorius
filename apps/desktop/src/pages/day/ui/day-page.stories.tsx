import type { Meta, StoryObj } from "@storybook/react-vite"
import { fn, within } from "storybook/test"
import { DayPage } from "@/pages/day/ui/day-page"
import {
  busyDay,
  emptyDay,
  sampleCommits,
  sampleDate,
  sampleDay,
} from "@/pages/day/ui/sample-day.fixture"

const meta = {
  title: "Apps/Desktop/Pages/Day",
  tags: ["autodocs"],
  component: DayPage,
  parameters: { layout: "fullscreen" },
  // The page fills the window in the app; give it a window-sized frame.
  decorators: [
    (Story) => (
      <div className="flex h-[820px] flex-col">
        <Story />
      </div>
    ),
  ],
  args: {
    date: sampleDate,
    today: "2026-10-11",
    state: { kind: "ready", timeline: sampleDay },
    onDateChange: fn(),
    zoom: 15,
    onZoomChange: fn(),
    view: "horizontal",
    onViewChange: fn(),
    unloggedMs: 172 * 60_000,
    nowMs: new Date(2026, 9, 9, 14, 32).getTime(),
    highlight: null,
    onClearHighlight: fn(),
    onMarkPrivate: fn(),
    CommitDetails: () => (
      <span className="flex flex-col gap-0.5 border-t border-border pt-2 text-xs text-t3">
        <span>
          4 files · <span className="text-operational">+86</span>{" "}
          <span className="text-critical">−12</span>
        </span>
        <span className="font-mono text-[11px] text-t4">
          crates/memorius-core/src/store.rs
        </span>
      </span>
    ),
    PromptText: () =>
      "Compare a hash of the normalized row instead of the raw JSON. Keep the existing tests.",
  },
} satisfies Meta<typeof DayPage>

export default meta

type Story = StoryObj<typeof meta>

export const Ready: Story = {}

export const BlockSelected: Story = {
  tags: ["visual-smoke"],
  play: async ({ canvas, userEvent }) => {
    await userEvent.click(
      canvas.getByRole("button", { name: /^OrderSync\.fs/ }),
    )
  },
}

export const WorkSelected: Story = {
  play: async ({ canvas, userEvent }) => {
    await userEvent.click(
      within(canvas.getByLabelText("Work context")).getByRole("button", {
        name: /^INT-290,/,
      }),
    )
  },
}

/** YouTrack's lane: logged, estimated and proposed blocks, and gaps. */
export const LoggedSelected: Story = {
  tags: ["visual-smoke"],
  args: { onOpenIssue: fn() },
  play: async ({ canvas, userEvent }) => {
    await userEvent.click(
      within(canvas.getByLabelText("YouTrack")).getByRole("button", {
        name: /^INT-177,/,
      }),
    )
  },
}

export const LoggedSelectedVertical: Story = {
  args: { view: "vertical", onOpenIssue: fn() },
  play: LoggedSelected.play,
}

export const AgentSelected: Story = {
  play: async ({ canvas, userEvent }) => {
    await userEvent.click(
      canvas.getByRole("button", { name: /^Claude Code, Order Search/ }),
    )
  },
}

export const WorkWithAgentSessions: Story = {
  play: async ({ canvas, userEvent }) => {
    await userEvent.click(canvas.getByRole("button", { name: /^IS-61\?, / }))
  },
}

export const PrivateAgentSelected: Story = {
  args: {
    state: {
      kind: "ready",
      timeline: {
        ...sampleDay,
        agents: sampleDay.agents.map((agent) =>
          agent.issue === "IS-61"
            ? {
                ...agent,
                private: true,
                split: { ...agent.split, agentMs: 0, uncountedMs: 90 * 60_000 },
              }
            : agent,
        ),
      },
    },
  },
  play: AgentSelected.play,
}

export const Vertical: Story = {
  args: { view: "vertical" },
  play: async ({ canvas, userEvent }) => {
    await userEvent.click(
      canvas.getByRole("button", { name: /^OrderSync\.fs/ }),
    )
  },
}

export const ManyAgents: Story = {
  args: { state: { kind: "ready", timeline: busyDay } },
}

export const ManyAgentsVertical: Story = {
  args: { state: { kind: "ready", timeline: busyDay }, view: "vertical" },
}

/** Design of the Commits lane (not fed by the app yet). */
const withCommits = {
  kind: "ready" as const,
  timeline: { ...sampleDay, commits: sampleCommits },
}

export const WithCommits: Story = {
  args: { state: withCommits },
}

export const WithCommitsVertical: Story = {
  args: { state: withCommits, view: "vertical" },
}

export const Today: Story = {
  args: { today: sampleDate },
}

export const IssueHighlighted: Story = {
  args: { today: sampleDate, highlight: "IS-61" },
}

export const Empty: Story = {
  args: { state: { kind: "ready", timeline: emptyDay } },
}

export const Loading: Story = {
  args: { state: { kind: "loading" } },
}

export const LoadError: Story = {
  args: { state: { kind: "error", message: "database is locked" } },
}
