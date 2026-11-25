import { afterEach, describe, expect, it, vi } from "vitest"
import { applyTheme, setThemePreference } from "@/shared/lib/theme"
import { readStoredTheme } from "@/shared/lib/theme-storage"

function systemDark(dark: boolean) {
  vi.stubGlobal(
    "matchMedia",
    vi.fn(() => ({
      matches: dark,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    })),
  )
}

afterEach(() => {
  localStorage.clear()
  document.documentElement.classList.remove("dark")
  vi.unstubAllGlobals()
})

describe("theme preference", () => {
  it("defaults to System", () => {
    expect(readStoredTheme()).toBe("system")
  })

  it("follows the system setting when the preference is System", () => {
    systemDark(true)
    applyTheme("system")
    expect(document.documentElement).toHaveClass("dark")
    systemDark(false)
    applyTheme("system")
    expect(document.documentElement).not.toHaveClass("dark")
  })

  it("keeps an explicit choice whatever the system says", () => {
    systemDark(true)
    setThemePreference("light")
    expect(readStoredTheme()).toBe("light")
    expect(document.documentElement).not.toHaveClass("dark")
  })

  it("forgets the saved value when System is chosen again", () => {
    setThemePreference("dark")
    setThemePreference("system")
    expect(localStorage.getItem("memorius.theme")).toBeNull()
  })
})
