import { useHotkey } from "@tanstack/react-hotkeys"
import { keepPreviousData, useQuery } from "@tanstack/react-query"
import { useNavigate } from "@tanstack/react-router"
import { SearchIcon } from "lucide-react"
import { useEffect, useState } from "react"
import { parseDay } from "@/app/shell/lib/parse-date"
import {
  type SearchTarget,
  dateGroups,
  resultGroups,
} from "@/app/shell/model/search-items"
import { SearchPalette, type SearchState } from "@/app/shell/ui/search-palette"
import { searchQuery } from "@/shared/api/memorius-commands"
import { todayIso } from "@/shared/lib/local-date"
import { Dialog, DialogContent, DialogTitle } from "@/shared/ui/dialog"

const DEBOUNCE_MS = 150
const isMac =
  typeof navigator !== "undefined" &&
  /Mac|iPhone|iPad/.test(navigator.userAgent)

/** The top-bar search: ⌘K (Ctrl+K on Windows) opens the palette anywhere. */
export function SearchButton() {
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState("")
  const [debounced, setDebounced] = useState("")
  const navigate = useNavigate()
  useHotkey("Mod+K", () => setOpen((value) => !value))
  useEffect(
    function debounceQuery() {
      const timer = setTimeout(() => setDebounced(query.trim()), DEBOUNCE_MS)
      return () => clearTimeout(timer)
    },
    [query],
  )
  // Each query is its own cache key, so a late answer never replaces a newer one.
  const results = useQuery({
    ...searchQuery(debounced),
    enabled: open && debounced.length >= 2,
    placeholderData: keepPreviousData,
    select: resultGroups,
  })

  const state: SearchState =
    query.trim().length < 2
      ? "idle"
      : results.isError
        ? "error"
        : results.isFetching || debounced !== query.trim()
          ? "loading"
          : "ready"
  const pick = (target: SearchTarget) => {
    setOpen(false)
    setQuery("")
    const search = target.issue ? { issue: target.issue } : {}
    void (target.page === "day"
      ? navigate({ to: "/day/$date", params: { date: target.date }, search })
      : navigate({ to: "/week/$week", params: { week: target.week }, search }))
  }
  return (
    <>
      <button
        type="button"
        onClick={() => setOpen(true)}
        aria-label={`Search, ${isMac ? "Command K" : "Control K"}`}
        className="glass inline-flex h-10 items-center gap-2 rounded-full pr-2 pl-3.5 text-t3 outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring max-[960px]:w-10 max-[960px]:justify-center max-[960px]:p-0"
      >
        <SearchIcon aria-hidden="true" className="size-4" />
        <span className="max-[960px]:hidden">Search</span>
        <kbd className="rounded-lg border border-input px-1.5 py-0.5 font-sans text-[11px] text-t4 max-[960px]:hidden">
          {isMac ? "⌘K" : "Ctrl+K"}
        </kbd>
      </button>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent
          showCloseButton={false}
          className="top-16 w-[660px] max-w-[calc(100%-20px)] translate-y-0 gap-0 overflow-hidden p-0 sm:max-w-[660px]"
        >
          <DialogTitle className="sr-only">Search</DialogTitle>
          <SearchPalette
            query={query}
            onQueryChange={setQuery}
            groups={results.data?.groups ?? []}
            dates={dateGroups(parseDay(query, todayIso()))}
            state={state}
            truncated={results.data?.truncated ?? false}
            onPick={pick}
          />
        </DialogContent>
      </Dialog>
    </>
  )
}
