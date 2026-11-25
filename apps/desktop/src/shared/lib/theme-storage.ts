import { z } from "zod"

export type ThemePreference = "system" | "light" | "dark"

/** Shared by the main window and the popover (same origin). */
export const THEME_STORAGE_KEY = "memorius.theme"
const storedThemeSchema = z.enum(["light", "dark"])

/** The saved choice; System when nothing (or nothing valid) is saved. */
export function readStoredTheme(): ThemePreference {
  try {
    return (
      storedThemeSchema.safeParse(localStorage.getItem(THEME_STORAGE_KEY))
        .data ?? "system"
    )
  } catch (error) {
    console.warn("Memorius could not read the stored theme", error)
    return "system"
  }
}

export function storeTheme(preference: ThemePreference) {
  try {
    if (preference === "system") localStorage.removeItem(THEME_STORAGE_KEY)
    else localStorage.setItem(THEME_STORAGE_KEY, preference)
  } catch (error) {
    // The choice then lasts until the window closes.
    console.warn("Memorius could not store the theme", error)
  }
}
