import type { AppRef } from "@/shared/api/generated/AppRef"
import type { SearchResults } from "@/shared/api/generated/SearchResults"
import { formatDuration } from "@/shared/lib/duration"
import {
  formatClock,
  formatLongDate,
  formatShortDate,
  todayIso,
} from "@/shared/lib/local-date"

/** Where a result opens: a day or a week, with one issue highlighted. */
export type SearchTarget =
  | { page: "day"; date: string; issue?: string }
  | { page: "week"; week: string; issue?: string }

type SearchChip = {
  label: string
  logged: boolean
  target: SearchTarget
}

type SearchItem = {
  key: string
  title: string
  meta: string
  right: string
  app?: AppRef
  glyph?: string
  chips: SearchChip[]
  target: SearchTarget
}

export type SearchGroup = { title: string; items: SearchItem[] }

const AGENT_APPS: Record<"claudeCode" | "codex", AppRef> = {
  claudeCode: { id: "com.anthropic.claudefordesktop", name: "Claude Code" },
  codex: { id: "com.openai.codex", name: "Codex" },
}

const dayOf = (ms: number) => todayIso(new Date(ms))
const ISSUE_KEY = /^[A-Z][A-Z0-9]+-\d+$/

/** Palette groups for stored results, as a query `select`. */
export function resultGroups(results: SearchResults) {
  return { groups: storedGroups(results), truncated: results.truncated }
}

function storedGroups(results: SearchResults): SearchGroup[] {
  const groups: SearchGroup[] = []
  if (results.issues.length > 0) {
    groups.push({
      title: "Issues",
      items: results.issues.map((hit) => {
        const byDay = new Map<string, { minutes: number; logged: boolean }>()
        for (const worklog of hit.worklogs) {
          const day = dayOf(worklog.startMs)
          const entry = byDay.get(day) ?? { minutes: 0, logged: true }
          byDay.set(day, {
            minutes: entry.minutes + worklog.minutes,
            logged: entry.logged && worklog.logged,
          })
        }
        const logged = hit.worklogs
          .filter((worklog) => worklog.logged)
          .reduce((sum, worklog) => sum + worklog.minutes, 0)
        const proposed = hit.worklogs
          .filter((worklog) => !worklog.logged)
          .reduce((sum, worklog) => sum + worklog.minutes, 0)
        const latest = [...byDay.keys()][0] ?? todayIso()
        return {
          key: `issue:${hit.issue}`,
          title: hit.issue,
          glyph: "#",
          meta: [
            logged > 0 && `${formatDuration(logged * 60_000)} logged`,
            proposed > 0 && `${formatDuration(proposed * 60_000)} proposed`,
          ]
            .filter(Boolean)
            .join(" · "),
          right: formatDuration((logged + proposed) * 60_000),
          chips: [...byDay.entries()].slice(0, 3).map(([day, entry]) => ({
            label: `${formatShortDate(new Date(`${day}T12:00:00`).getTime())} · ${formatDuration(entry.minutes * 60_000)}`,
            logged: entry.logged,
            target: { page: "day", date: day, issue: hit.issue },
          })),
          target: { page: "week", week: latest, issue: hit.issue },
        }
      }),
    })
  }
  if (results.windows.length > 0) {
    groups.push({
      title: "Apps and windows",
      items: results.windows.map((hit) => {
        const days = [...new Set(hit.spans.map(([start]) => dayOf(start)))]
        const [start, end] = hit.spans[0] ?? [0, 0]
        const issue = ISSUE_KEY.exec(hit.title.split(/\s/)[0] ?? "")?.[0]
        return {
          key: `window:${hit.app.id}:${hit.title}`,
          title: hit.title,
          app: hit.app,
          meta: `${hit.app.name} · last ${formatShortDate(start)} ${formatClock(start)}–${formatClock(end)}${days.length > 1 ? ` · ${days.length} days` : ""}`,
          right: formatDuration(hit.totalMs),
          chips: [],
          target: {
            page: "day",
            date: dayOf(start),
            ...(issue ? { issue } : {}),
          },
        }
      }),
    })
  }
  if (results.sessions.length > 0) {
    groups.push({
      title: "Agent sessions",
      items: results.sessions.map((hit) => ({
        key: `session:${hit.source}:${hit.startMs}`,
        title: hit.title ?? hit.branch ?? AGENT_APPS[hit.source].name,
        app: AGENT_APPS[hit.source],
        meta: [
          AGENT_APPS[hit.source].name,
          `${formatShortDate(hit.startMs)} ${formatClock(hit.startMs)}–${formatClock(hit.endMs)}`,
          hit.branch,
        ]
          .filter(Boolean)
          .join(" · "),
        right: formatDuration(hit.endMs - hit.startMs),
        chips: [],
        target: { page: "day", date: dayOf(hit.startMs) },
      })),
    })
  }
  return groups
}

/** The typed date, if the query reads as one. */
export function dateGroups(date: string | undefined): SearchGroup[] {
  const groups: SearchGroup[] = []
  if (date) {
    groups.push({
      title: "Go to date",
      items: [
        {
          key: `date:${date}`,
          title: formatLongDate(date),
          glyph: String(Number(date.slice(8))),
          meta: date === todayIso() ? "Today · open the day" : "Open the day",
          right: "",
          chips: [],
          target: { page: "day", date },
        },
      ],
    })
  }
  return groups
}
