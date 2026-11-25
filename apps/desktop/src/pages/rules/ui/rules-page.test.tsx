import { fireEvent, render, screen, within } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { RulesPage } from "@/pages/rules/ui/rules-page"

const apps = [
  {
    app: { id: "com.microsoft.teams2", name: "Microsoft Teams" },
    durationMs: 60_000,
  },
  {
    app: { id: "net.whatsapp.WhatsApp", name: "WhatsApp" },
    durationMs: 60_000,
  },
]

describe("RulesPage", () => {
  it("makes a recent app private", () => {
    const onAdd = vi.fn()
    render(
      <RulesPage
        rules={[]}
        apps={apps}
        matches={undefined}
        loading={false}
        onAdd={onAdd}
        onRemove={vi.fn()}
      />,
    )
    const section = screen.getByRole("region", { name: "Private apps" })

    fireEvent.change(within(section).getByRole("combobox"), {
      target: { value: "net.whatsapp.WhatsApp" },
    })
    fireEvent.click(
      within(section).getByRole("button", { name: "Make private" }),
    )

    expect(onAdd).toHaveBeenCalledWith({
      kind: "privateApp",
      appId: "net.whatsapp.WhatsApp",
    })
  })

  it("adds a website rule only with a valid issue key", () => {
    const onAdd = vi.fn()
    render(
      <RulesPage
        rules={[]}
        apps={apps}
        matches={undefined}
        loading={false}
        onAdd={onAdd}
        onRemove={vi.fn()}
      />,
    )
    const section = screen.getByRole("region", { name: "Issue rules" })
    const add = within(section).getByRole("button", { name: "Add rule" })

    fireEvent.change(within(section).getByLabelText("Website host"), {
      target: { value: "orders.example.com" },
    })
    fireEvent.change(within(section).getByLabelText("Issue"), {
      target: { value: "SEL" },
    })
    expect(add).toBeDisabled()
    fireEvent.change(within(section).getByLabelText("Issue"), {
      target: { value: "SEL-54" },
    })
    fireEvent.click(add)

    expect(onAdd).toHaveBeenCalledWith({
      kind: "hostIssue",
      host: "orders.example.com",
      issue: "SEL-54",
    })
  })

  it("removes a rule", () => {
    const onRemove = vi.fn()
    render(
      <RulesPage
        rules={[
          {
            id: 7,
            rule: {
              kind: "titleIssue",
              text: "Daily stand-up",
              issue: "SEL-21",
            },
          },
        ]}
        apps={apps}
        matches={new Map([[7, 45 * 60_000]])}
        loading={false}
        onAdd={vi.fn()}
        onRemove={onRemove}
      />,
    )

    expect(
      screen.getByRole("row", { name: /Daily stand-up/ }),
    ).toHaveTextContent("45m")
    fireEvent.click(
      screen.getByRole("button", { name: "Remove rule Daily stand-up" }),
    )

    expect(onRemove).toHaveBeenCalledWith(7)
  })
})
