import { MonitorIcon, MoonIcon, SunIcon, type LucideIcon } from "lucide-react"
import { setThemePreference, useThemePreference } from "@/shared/lib/theme"
import type { ThemePreference } from "@/shared/lib/theme-storage"
import { ToggleGroup, ToggleGroupItem } from "@/shared/ui/toggle-group"

const OPTIONS: readonly [ThemePreference, string, LucideIcon][] = [
  ["system", "Use the system theme", MonitorIcon],
  ["light", "Light theme", SunIcon],
  ["dark", "Dark theme", MoonIcon],
]

function isPreference(value: string | undefined): value is ThemePreference {
  return value === "system" || value === "light" || value === "dark"
}

/** System, Light or Dark; System follows the OS setting. */
export function ThemeSwitch() {
  const preference = useThemePreference()
  return (
    <ToggleGroup
      aria-label="Theme"
      value={[preference]}
      onValueChange={(value) => {
        const next = value[0]
        if (isPreference(next)) setThemePreference(next)
      }}
      className="glass rounded-full p-[3px]"
    >
      {OPTIONS.map(([value, label, Icon]) => (
        <ToggleGroupItem
          key={value}
          value={value}
          aria-label={label}
          title={label}
          className="size-8 min-w-8 rounded-full px-0"
        >
          <Icon className="size-3.5" />
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  )
}
