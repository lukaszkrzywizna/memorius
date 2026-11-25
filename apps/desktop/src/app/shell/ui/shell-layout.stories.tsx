import type { Meta, StoryObj } from "@storybook/react-vite"
import { createElement } from "react"
import { fn } from "storybook/test"
import { NeedsYouPopover } from "@/app/shell/ui/needs-you-button"
import { sampleNeeds } from "@/app/shell/ui/needs-you.fixture"
import { ShellLayout } from "@/app/shell/ui/shell-layout"

/** The layout names each link, so the anchors have no content here. */
const link = (href: string) => createElement("a", { href })

const meta = {
  title: "Apps/Desktop/Shell/Shell Layout",
  tags: ["autodocs"],
  component: ShellLayout,
  parameters: { layout: "fullscreen" },
  args: {
    current: "day",
    waiting: 4,
    status: {
      state: {
        kind: "recording",
        current: {
          app: { id: "com.jetbrains.rider", name: "Rider" },
          title: { kind: "known", text: "OrderSync.fs — Shop" },
          document: null,
          url: null,
        },
      },
      titleAccess: "granted",
      titlesLostMs: null,
    },
    links: {
      day: link("#/day"),
      week: link("#/week"),
      queue: link("#/queue"),
      rules: link("#/rules"),
      settings: link("#/settings"),
    },
    needs: <NeedsYouPopover needs={sampleNeeds} onOpenQueue={fn()} />,
    search: (
      <button
        type="button"
        className="glass inline-flex h-10 items-center gap-2 rounded-full px-3.5 text-t3"
      >
        Search
      </button>
    ),
    children: (
      <div className="glass grid h-96 place-items-center rounded-[20px] text-t4">
        Page
      </div>
    ),
  },
} satisfies Meta<typeof ShellLayout>

export default meta

type Story = StoryObj<typeof meta>

export const OnDay: Story = { tags: ["visual-smoke"] }

export const NeedsYouOpen: Story = {
  args: {
    needs: (
      <NeedsYouPopover needs={sampleNeeds} onOpenQueue={fn()} defaultOpen />
    ),
  },
}

export const Paused: Story = {
  args: {
    current: "settings",
    waiting: 0,
    status: {
      state: { kind: "paused", untilMs: new Date(2026, 9, 9, 14).getTime() },
      titleAccess: "granted",
      titlesLostMs: null,
    },
  },
}
