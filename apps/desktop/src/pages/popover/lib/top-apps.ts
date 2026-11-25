import type { AppRef } from "@/shared/api/generated/AppRef"
import type { Block } from "@/shared/api/generated/Block"

export type AppTotal = { app: AppRef; durationMs: number }

/** Time per app across the day, longest first. */
export function topApps(blocks: readonly Block[], limit = 7): AppTotal[] {
  const totals = new Map<string, AppTotal>()
  for (const block of blocks) {
    if (block.kind !== "apps") continue
    for (const item of block.items) {
      const total = totals.get(item.app.id) ?? { app: item.app, durationMs: 0 }
      total.durationMs += item.durationMs
      totals.set(item.app.id, total)
    }
  }
  return [...totals.values()]
    .sort((a, b) => b.durationMs - a.durationMs)
    .slice(0, limit)
}
