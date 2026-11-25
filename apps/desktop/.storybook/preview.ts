import type { Preview } from "@storybook/react-vite"
import { createElement } from "react"
import { applyTheme } from "@/shared/lib/theme"
import "@/styles.css"

const preview: Preview = {
  parameters: {
    a11y: { test: "error" },
  },
  globalTypes: {
    theme: {
      description: "Light or dark theme",
      toolbar: { title: "Theme", icon: "mirror", items: ["light", "dark"] },
    },
  },
  initialGlobals: { theme: "dark" },
  // The glass surfaces need the app's backdrop behind them.
  decorators: [
    (Story, context) => {
      applyTheme(context.globals.theme === "light" ? "light" : "dark")
      return createElement(
        "div",
        { className: "min-h-screen bg-(--backdrop) p-4 text-foreground" },
        createElement(Story),
      )
    },
  ],
}

export default preview
