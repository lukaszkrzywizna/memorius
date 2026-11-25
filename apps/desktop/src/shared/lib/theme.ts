import { useSyncExternalStore } from "react"
import {
  readStoredTheme,
  storeTheme,
  THEME_STORAGE_KEY,
  type ThemePreference,
} from "@/shared/lib/theme-storage"

const DARK_QUERY = "(prefers-color-scheme: dark)"
let current: ThemePreference = "system"
const listeners = new Set<() => void>()

function notify() {
  for (const listener of listeners) listener()
}

function systemIsDark() {
  return typeof matchMedia === "function" && matchMedia(DARK_QUERY).matches
}

/** Puts `.dark` on <html> for the preference and the current system setting. */
export function applyTheme(preference: ThemePreference) {
  const dark =
    preference === "dark" || (preference === "system" && systemIsDark())
  document.documentElement.classList.toggle("dark", dark)
}

export function setThemePreference(preference: ThemePreference) {
  storeTheme(preference)
  current = preference
  applyTheme(preference)
  notify()
}

/**
 * Applies the saved theme now and follows later changes: the system setting
 * and a choice made in another Memorius window (main window or popover).
 */
export function startTheme() {
  current = readStoredTheme()
  applyTheme(current)
  if (typeof matchMedia === "function") {
    matchMedia(DARK_QUERY).addEventListener("change", () => applyTheme(current))
  }
  addEventListener("storage", (event) => {
    if (event.key !== THEME_STORAGE_KEY && event.key !== null) return
    current = readStoredTheme()
    applyTheme(current)
    notify()
  })
}

function subscribe(listener: () => void) {
  listeners.add(listener)
  return () => listeners.delete(listener)
}

/** The chosen theme, updated when this or another window changes it. */
export function useThemePreference(): ThemePreference {
  return useSyncExternalStore(subscribe, () => current)
}
