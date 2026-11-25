import "@testing-library/jest-dom/vitest"
import { cleanup } from "@testing-library/react"
import { afterEach } from "vitest"

// Tests read clocks as a 24-hour system shows them.
Object.defineProperty(window.navigator, "language", {
  value: "en-GB",
  configurable: true,
})

afterEach(() => {
  cleanup()
})
