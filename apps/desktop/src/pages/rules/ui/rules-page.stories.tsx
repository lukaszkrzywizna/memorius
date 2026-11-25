import type { Meta, StoryObj } from "@storybook/react-vite"
import { fn } from "storybook/test"
import { RulesPage } from "@/pages/rules/ui/rules-page"

const minutes = (count: number) => count * 60_000

const meta = {
  title: "Apps/Desktop/Pages/Rules",
  tags: ["autodocs"],
  component: RulesPage,
  parameters: { layout: "fullscreen" },
  args: {
    loading: false,
    matches: new Map([
      [2, minutes(130)],
      [3, minutes(150)],
    ]),
    rules: [
      { id: 1, rule: { kind: "privateApp", appId: "net.whatsapp.WhatsApp" } },
      {
        id: 2,
        rule: {
          kind: "hostIssue",
          host: "orders.example.com",
          issue: "SEL-54",
        },
      },
      {
        id: 3,
        rule: { kind: "titleIssue", text: "Daily stand-up", issue: "SEL-21" },
      },
    ],
    apps: [
      {
        app: { id: "com.jetbrains.rider", name: "Rider" },
        durationMs: minutes(1210),
      },
      {
        app: { id: "company.thebrowser.dia", name: "Dia" },
        durationMs: minutes(640),
      },
      {
        app: { id: "com.microsoft.teams2", name: "Microsoft Teams" },
        durationMs: minutes(300),
      },
      {
        app: { id: "net.whatsapp.WhatsApp", name: "WhatsApp" },
        durationMs: minutes(42),
      },
    ],
    onAdd: fn(),
    onRemove: fn(),
  },
} satisfies Meta<typeof RulesPage>

export default meta

type Story = StoryObj<typeof meta>

export const WithRules: Story = {
  tags: ["visual-smoke"],
}

export const Empty: Story = {
  args: { rules: [] },
}

export const Loading: Story = {
  args: { loading: true },
}
