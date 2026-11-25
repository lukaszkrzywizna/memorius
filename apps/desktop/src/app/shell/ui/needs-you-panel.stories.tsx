import type { Meta, StoryObj } from "@storybook/react-vite"
import { fn } from "storybook/test"
import { NeedsYouPanel } from "@/app/shell/ui/needs-you-panel"
import { nothingNeeded, sampleNeeds } from "@/app/shell/ui/needs-you.fixture"

const meta = {
  title: "Apps/Desktop/Shell/Needs You Panel",
  tags: ["autodocs"],
  component: NeedsYouPanel,
  decorators: [
    (Story) => (
      <div className="frost w-[340px] rounded-[20px] p-4">
        <Story />
      </div>
    ),
  ],
  args: {
    needs: sampleNeeds,
    onOpenQueue: fn(),
  },
} satisfies Meta<typeof NeedsYouPanel>

export default meta

type Story = StoryObj<typeof meta>

export const Everything: Story = {
  tags: ["visual-smoke"],
}

export const NothingNeeded: Story = {
  args: { needs: nothingNeeded },
}
