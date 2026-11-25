import { SearchIcon } from "lucide-react"
import { useState } from "react"
import { AppIcon } from "@/entities/activity"
import type { SearchGroup, SearchTarget } from "@/app/shell/model/search-items"
import { LoadingRegion, Skeleton } from "@/shared/ui/skeleton"

export type SearchState = "idle" | "loading" | "error" | "ready"

type SearchPaletteProps = {
  query: string
  onQueryChange: (query: string) => void
  /** Stored results; ignored until the query is long enough. */
  groups: SearchGroup[]
  /** The typed date, when the query reads as one. */
  dates: SearchGroup[]
  state: SearchState
  truncated: boolean
  onPick: (target: SearchTarget) => void
}

/** "When did I work on this?": issues, windows, sessions and dates. */
export function SearchPalette({
  query,
  onQueryChange,
  groups: stored,
  dates,
  state,
  truncated,
  onPick,
}: SearchPaletteProps) {
  const groups = [...(state === "idle" ? [] : stored), ...dates]
  const items = groups.flatMap((group) => group.items)
  const [active, setActive] = useState(0)
  const current = Math.min(active, Math.max(0, items.length - 1))
  const starts = groups.map((_, groupIndex) =>
    groups
      .slice(0, groupIndex)
      .reduce((count, group) => count + group.items.length, 0),
  )
  return (
    <div className="flex max-h-[min(560px,calc(100dvh-120px))] flex-col">
      <div className="flex h-13 items-center gap-2.5 border-b border-border px-3.5 text-t3">
        <SearchIcon aria-hidden="true" className="size-4" />
        <input
          role="combobox"
          aria-expanded={items.length > 0}
          aria-controls="search-results"
          aria-activedescendant={
            items.length > 0 ? `search-${current}` : undefined
          }
          aria-label="Search issues, apps, windows, sessions or a date"
          placeholder="Issue, app, window, session or date…"
          value={query}
          onChange={(event) => {
            onQueryChange(event.target.value)
            setActive(0)
          }}
          onKeyDown={(event) => {
            if (event.key === "ArrowDown") {
              event.preventDefault()
              setActive(Math.min(current + 1, items.length - 1))
            } else if (event.key === "ArrowUp") {
              event.preventDefault()
              setActive(Math.max(current - 1, 0))
            } else if (event.key === "Enter") {
              const item = items[current]
              if (item) {
                event.preventDefault()
                onPick(item.target)
              }
            }
          }}
          className="h-10 min-w-0 flex-1 border-0 bg-transparent text-[15px] text-t1 outline-none placeholder:text-t4"
        />
        <kbd className="rounded-lg border border-input px-1.5 py-0.5 font-sans text-[11px] text-t4">
          esc
        </kbd>
      </div>
      <div
        id="search-results"
        role="listbox"
        aria-label="Results"
        className="min-h-0 overflow-y-auto py-1 pb-2 [scrollbar-width:thin]"
      >
        {groups.map((group, groupIndex) => (
          <div key={group.title} role="group" aria-label={group.title}>
            <div className="px-4.5 pt-2.5 pb-1 text-xs text-t4">
              {group.title}
            </div>
            {group.items.map((item, itemIndex) => {
              const position = (starts[groupIndex] ?? 0) + itemIndex
              return (
                <div
                  key={item.key}
                  id={`search-${position}`}
                  role="option"
                  aria-selected={position === current}
                  tabIndex={-1}
                  onClick={() => onPick(item.target)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter") onPick(item.target)
                  }}
                  onMouseMove={() => setActive(position)}
                  className={`mx-1.5 flex cursor-pointer items-center gap-2.5 rounded-xl px-2.5 py-2 ${
                    position === current ? "bg-accent" : ""
                  }`}
                >
                  {item.app ? (
                    <AppIcon app={item.app} size={18} />
                  ) : (
                    <span
                      aria-hidden="true"
                      className="grid size-[18px] shrink-0 place-items-center rounded-[5px] bg-accent text-[10px] text-t2"
                    >
                      {item.glyph}
                    </span>
                  )}
                  <span className="flex min-w-0 flex-1 flex-col gap-0.5">
                    <span className="truncate text-t1">
                      <Highlighted text={item.title} query={query} />
                    </span>
                    <span className="truncate text-xs text-t4">
                      {item.meta}
                    </span>
                  </span>
                  {item.chips.length > 0 && (
                    <span className="flex gap-1 max-[960px]:hidden">
                      {item.chips.map((chip) => (
                        <button
                          key={chip.label}
                          type="button"
                          title={
                            chip.logged
                              ? "Logged in YouTrack"
                              : "Proposed or approved"
                          }
                          onClick={(event) => {
                            event.stopPropagation()
                            onPick(chip.target)
                          }}
                          className={`h-6 rounded-full border px-2 text-[11px] whitespace-nowrap text-t2 outline-none focus-visible:ring-2 focus-visible:ring-ring ${
                            chip.logged ? "border-input" : "border-attention/70"
                          }`}
                        >
                          {chip.label}
                        </button>
                      ))}
                    </span>
                  )}
                  <span className="shrink-0 whitespace-nowrap text-t2">
                    {item.right}
                  </span>
                </div>
              )
            })}
          </div>
        ))}
        <SearchStatus
          state={state}
          query={query}
          empty={items.length === 0}
          truncated={truncated}
        />
      </div>
      <div className="flex gap-4 border-t border-border px-4.5 py-2 text-[11px] text-t5 max-[960px]:hidden">
        <span>↑ ↓ move</span>
        <span>↵ open</span>
        <span>esc close</span>
        <span className="ml-auto">
          Last 90 days · prompt text is not searched
        </span>
      </div>
    </div>
  )
}

function SearchStatus({
  state,
  query,
  empty,
  truncated,
}: {
  state: SearchState
  query: string
  empty: boolean
  truncated: boolean
}) {
  if (state === "loading" && empty) {
    return (
      <LoadingRegion
        label="Searching"
        className="flex flex-col gap-1 px-4 py-2"
      >
        {[0, 1, 2].map((row) => (
          <div key={row} className="flex items-center gap-2.5 px-1 py-2">
            <Skeleton className="size-4.5 rounded-[5px]" />
            <span className="flex flex-1 flex-col gap-1.5">
              <Skeleton className="h-3.5 w-2/5" />
              <Skeleton className="h-3 w-3/5" />
            </span>
            <Skeleton className="h-3.5 w-12" />
          </div>
        ))}
      </LoadingRegion>
    )
  }
  const text =
    state === "error"
      ? "Search failed. Try again."
      : state === "idle"
        ? "Type an issue key, an app, a window title, a session or a date."
        : empty
          ? `No match for “${query.trim()}” in the last 90 days.`
          : truncated
            ? "More matches exist: type more to narrow them."
            : ""
  return text ? (
    <p role="status" className="m-0 px-4.5 py-3 text-[13px] text-t4">
      {text}
    </p>
  ) : null
}

function Highlighted({ text, query }: { text: string; query: string }) {
  const needle = query.trim().toLowerCase()
  const at = needle.length >= 2 ? text.toLowerCase().indexOf(needle) : -1
  if (at < 0) return text
  return (
    <>
      {text.slice(0, at)}
      <span className="font-medium text-foreground">
        {text.slice(at, at + needle.length)}
      </span>
      {text.slice(at + needle.length)}
    </>
  )
}
