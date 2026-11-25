import type { Meta, StoryObj } from "@storybook/react-vite"
import { fn } from "storybook/test"
import { AttentionBanner } from "@/app/shell/ui/attention-banner"

const meta = {
  title: "Apps/Desktop/Shell/AttentionBanner",
  tags: ["autodocs"],
  component: AttentionBanner,
  args: {
    tone: "critical",
    title: "Memorius can't read window titles.",
    text: "Accessibility was switched off at 14:02; since then only app names are recorded.",
    action: "Fix",
    onAction: fn(),
  },
} satisfies Meta<typeof AttentionBanner>

export default meta
type Story = StoryObj<typeof meta>

export const TitlesLost: Story = { tags: ["visual-smoke"] }

export const AgentSignedOut: Story = {
  args: {
    tone: "attention",
    title: "Claude Code is signed out.",
    text: "The daily proposals run cannot start.",
    action: "Sign in again",
  },
}
