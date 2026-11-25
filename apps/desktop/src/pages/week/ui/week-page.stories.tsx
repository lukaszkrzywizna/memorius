import type { Meta, StoryObj } from "@storybook/react-vite"
import { fn } from "storybook/test"
import { WeekPage } from "@/pages/week/ui/week-page"
import { sampleWeek } from "@/pages/week/ui/sample-week.fixture"

const meta = {
  title: "Apps/Desktop/Pages/Week",
  tags: ["autodocs"],
  component: WeekPage,
  parameters: { layout: "fullscreen" },
  args: {
    week: "2026-10-09",
    today: "2026-10-11",
    state: { kind: "ready", summary: sampleWeek },
    onWeekChange: fn(),
    onOpenDay: fn(),
    highlight: null,
  },
} satisfies Meta<typeof WeekPage>

export default meta

type Story = StoryObj<typeof meta>

export const Ready: Story = {
  tags: ["visual-smoke"],
}

export const NoIssues: Story = {
  args: { state: { kind: "ready", summary: { ...sampleWeek, issues: [] } } },
}

export const IssueHighlighted: Story = {
  args: { highlight: "IS-61" },
}

export const EarlierWeek: Story = {
  args: { today: "2026-10-22" },
}

export const Loading: Story = {
  args: { state: { kind: "loading" } },
}
