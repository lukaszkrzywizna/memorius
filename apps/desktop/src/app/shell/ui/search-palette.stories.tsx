import type { Meta, StoryObj } from "@storybook/react-vite"
import { fn } from "storybook/test"
import { dateGroups, resultGroups } from "@/app/shell/model/search-items"
import { sampleSearch } from "@/app/shell/ui/search.fixture"
import { SearchPalette } from "@/app/shell/ui/search-palette"

const meta = {
  title: "Apps/Desktop/Shell/Search Palette",
  tags: ["autodocs"],
  component: SearchPalette,
  decorators: [
    (Story) => (
      <div className="frost mx-auto mt-10 w-[660px] overflow-hidden rounded-[20px]">
        <Story />
      </div>
    ),
  ],
  args: {
    query: "is-61",
    onQueryChange: fn(),
    groups: resultGroups(sampleSearch).groups,
    dates: [],
    state: "ready",
    truncated: false,
    onPick: fn(),
  },
} satisfies Meta<typeof SearchPalette>

export default meta

type Story = StoryObj<typeof meta>

export const Results: Story = { tags: ["visual-smoke"] }

export const WithDate: Story = {
  args: { query: "28 sep", groups: [], dates: dateGroups("2026-09-28") },
}

export const NoMatch: Story = {
  args: { query: "auth0", groups: [] },
}

export const BeforeTyping: Story = {
  args: { query: "", groups: [], state: "idle" },
}

export const Searching: Story = {
  args: { groups: [], state: "loading" },
}
