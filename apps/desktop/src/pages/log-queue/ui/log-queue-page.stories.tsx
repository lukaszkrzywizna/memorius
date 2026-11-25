import type { Meta, StoryObj } from "@storybook/react-vite"
import { fn } from "storybook/test"
import { LogQueuePage } from "@/pages/log-queue/ui/log-queue-page"
import {
  doneRun,
  failedRun,
  runningRun,
  sampleAgentSettings,
} from "@/pages/log-queue/ui/sample-agent.fixture"
import { sampleQueue } from "@/pages/log-queue/ui/sample-queue.fixture"

const meta = {
  title: "Apps/Desktop/Pages/Log Queue",
  tags: ["autodocs"],
  component: LogQueuePage,
  parameters: { layout: "fullscreen" },
  // The page fills the window in the app; give it a window-sized frame.
  decorators: [
    (Story) => (
      <div className="flex h-[760px] flex-col">
        <Story />
      </div>
    ),
  ],
  args: {
    items: sampleQueue,
    onApprove: fn(),
    onReject: fn(),
    onUndo: fn(),
    onApproveMany: fn(),
    bulk: { pending: false, failed: [] },
    daySummaries: [
      {
        date: "2026-10-09",
        workMs: 7 * 3_600_000,
        loggedMs: 30 * 60_000,
        proposedMs: 3 * 3_600_000,
        unloggedMs: 3.5 * 3_600_000,
      },
    ],
    workTypes: [
      "Development",
      "Testing",
      "Documentation",
      "Investigation",
      "Implementation",
    ],
    agent: {
      settings: sampleAgentSettings,
      runs: [doneRun],
      onPrepare: fn(),
      onStop: fn(),
    },
    youtrack: {
      onLog: fn(),
      connected: true,
      pending: false,
      result: undefined,
      error: undefined,
    },
  },
} satisfies Meta<typeof LogQueuePage>

export default meta

type Story = StoryObj<typeof meta>

export const WithProposals: Story = {
  tags: ["visual-smoke"],
}

export const DetailsOpen: Story = {
  play: async ({ canvas, userEvent }) => {
    await userEvent.click(
      canvas.getByRole("button", { name: /^Details of IS-61/ }),
    )
  },
}

export const BulkFailed: Story = {
  args: {
    bulk: {
      pending: false,
      failed: [{ id: 2, issue: "IS-61", message: "database is locked" }],
    },
  },
}

export const AgentRunning: Story = {
  args: {
    agent: {
      settings: sampleAgentSettings,
      runs: [runningRun, doneRun],
      onPrepare: fn(),
      onStop: fn(),
    },
  },
}

export const AgentFailed: Story = {
  args: {
    agent: {
      settings: sampleAgentSettings,
      runs: [failedRun, doneRun],
      onPrepare: fn(),
      onStop: fn(),
    },
  },
}

export const Empty: Story = {
  args: { items: [] },
}

export const EmptyWhileRunning: Story = {
  args: {
    items: [],
    agent: {
      settings: sampleAgentSettings,
      runs: [runningRun],
      onPrepare: fn(),
      onStop: fn(),
    },
  },
}

export const Loading: Story = {
  args: { items: undefined },
}
