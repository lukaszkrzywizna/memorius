import type { Meta, StoryObj } from "@storybook/react-vite"
import { fn } from "storybook/test"
import type { SetupStep } from "@/pages/setup/lib/setup-steps"
import { SetupPage } from "@/pages/setup/ui/setup-page"

const steps = (titles: boolean): SetupStep[] => [
  { id: "notifications", required: false, done: true },
  { id: "titles", required: true, done: titles },
  { id: "login", required: false, done: titles },
  { id: "repositories", required: false, done: titles },
  { id: "agent", required: false, done: false },
  { id: "commandLine", required: false, done: false },
  { id: "tracker", required: false, done: false },
]

const action = (label: string) => ({ label, onClick: fn() })

const meta = {
  title: "Apps/Desktop/Pages/Setup",
  tags: ["autodocs"],
  component: SetupPage,
  parameters: { layout: "fullscreen" },
  args: {
    steps: steps(false),
    notes: {
      notifications: "Allowed",
      agent: "Claude Code found, signed out.",
    },
    actions: {
      notifications: [action("Send a test"), action("It arrived")],
      titles: [action("Open System Settings")],
      login: [action("Turn on"), action("Open Login Items")],
      repositories: [],
      agent: [action("Sign in")],
      commandLine: [action("Install command"), action("Add to Claude Code")],
      tracker: [action("Connect")],
    },
    waiting: null,
    folders: { suggestions: ["~/work"], onAdd: fn() },
    onFinish: fn(),
  },
} satisfies Meta<typeof SetupPage>

export default meta
type Story = StoryObj<typeof meta>

export const FirstLaunch: Story = { tags: ["visual-smoke"] }

export const WaitingForAccessibility: Story = {
  args: {
    waiting: "titles",
    actions: {
      ...meta.args.actions,
      titles: [action("Open System Settings"), action("Restart Memorius")],
    },
  },
}

export const RequiredDone: Story = {
  args: {
    steps: steps(true),
    notes: { ...meta.args.notes, titles: "Allowed", login: "On" },
  },
}
