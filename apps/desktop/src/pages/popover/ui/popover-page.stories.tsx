import type { Meta, StoryObj } from "@storybook/react-vite"
import { fn } from "storybook/test"
import { PopoverPage } from "@/pages/popover/ui/popover-page"
import { sampleToday } from "@/pages/popover/ui/sample-today.fixture"

const meta = {
  title: "Apps/Desktop/Pages/Popover",
  tags: ["autodocs"],
  component: PopoverPage,
  parameters: { layout: "centered" },
  decorators: [
    (Story) => (
      <div style={{ width: 360, height: 560 }}>
        <Story />
      </div>
    ),
  ],
  args: {
    status: {
      state: {
        kind: "recording",
        current: {
          app: { id: "com.jetbrains.rider", name: "Rider" },
          title: { kind: "known", text: "CatalogSource.fs — Shop" },
          document: null,
          url: null,
        },
      },
      titleAccess: "granted",
      titlesLostMs: null,
    },
    today: sampleToday,
    onOpen: fn(),
    onPause: fn(),
    onResume: fn(),
    onQuit: fn(),
  },
} satisfies Meta<typeof PopoverPage>

export default meta

type Story = StoryObj<typeof meta>

export const Recording: Story = {
  tags: ["visual-smoke"],
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
