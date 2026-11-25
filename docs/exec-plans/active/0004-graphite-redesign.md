# Plan 0004 — Graphite redesign of the desktop UI

Status: accepted by Łukasz 2026-10-15. Phases 1–7 done. Waiting for Łukasz's review of the installed build.

Screens: the private design canvas, page "Wild looks":
"Living day prototype" (horizontal and vertical) and "Living prototype ·
phone" are the reference for every screen; "App icon" holds the Tick dial.
The prototype source is the canvas, not this repository.

## Why

The shadcn + glass migration (preset `b39h12fjM`, lime/ink, pastel backdrop)
shipped in 5f7b364. Łukasz then chose a different look in the canvas: the
calm graphite language of the Vexto shot, light and dark, a top bar instead
of the rail, both timeline directions, a dense desktop Log queue and a search
palette. The preset tokens and the glass layer do not fit it, so the theme is
rebuilt from scratch and every screen is restyled.

## Decisions already made (canvas sessions, 2026-10-15)

- Look: near-black graphite (dark) and soft white (light) surfaces, calm
  frosted glass, no glow, no corner shine, no colored blobs. Backdrop: one solid
  colour (the contour circles were removed on 2026-10-29 at Łukasz's request).
- Color: grayscale. Accents only Attention `#F39444` (needs you, proposed),
  Operational `#5EC557` (recording, logged), Critical `#FF4747` (errors).
  Small square dots, not round ones, for status.
- Type: Helvetica Neue stack (`"Helvetica Neue", Helvetica, Arial,
sans-serif`; Windows falls back to Arial). Weight 300 only at 22 px and up;
  400 below. Tabular numbers. Manrope is removed.
- Shadows: visible in dark mode, almost none in light mode.
- Theme: System / Light / Dark radio in the top bar; System is the default.
- Shell: top bar = Tick dial logo, tabs (Day, Week, Log queue with count,
  Apps & rules), Search ⌘K, recording pill, Needs you icon (tray + count),
  theme radio. The rail goes away.
- Needs you: hidden by default; the top-bar icon opens a floating panel on
  every tab. The Day header shows only the totals.
- Day: one floating details card (bottom sheet on narrow widths), zoom bar
  with the horizontal/vertical switch and "now"; no link dot in vertical.
- Week: ← / → and "This week"; day cards; timesheet with an optional
  highlighted issue.
- Log queue: dense table (44 px rows, group divider rows, filters, bulk
  approve, row actions) and a details pane with inline edit.
- Apps & rules: two-column settings layout, max width 1080 px.
- Search: ⌘K / Ctrl+K palette for "when did I work on this?" (see phase 6).
- Width tiers (container queries on the shell): ≤ 960 px = compact top bar,
  stacked Apps & rules, two-line queue rows + details drawer; ≤ 720 px =
  bottom tab bar, 2×2 totals, sheets.
- App icon: Tick dial on the graphite tile for the Dock and installers.

## Phases

Each phase ends with green `pnpm agent:preflight`, Storybook stories for the
new states (light and dark), `devtools-validation-loop` at 1440, 780 and
390 px, and one commit (or a few small ones).

1. **Theme foundation.** Replace the preset tokens and the glass layer in
   `styles.css` with graphite tokens for `:root` (light) and `.dark`, mapped
   onto the shadcn variables (`--background`, `--card`, `--popover`,
   `--primary`, `--muted-foreground`, `--border`, `--ring`, …) plus our own
   (`--attention`, `--operational`, `--critical`, text steps `--t1…--t5`,
   `--glass`, `--frost`, `--warm`). New utilities: `glass`, `frost`,
   `warm-panel`, `contour-backdrop`, `ticks`. Restyle the vendored shadcn
   components through their `*-variants.ts` (pill buttons, toggle group,
   table, input, select, badge, tooltip, card). Theme preference: a small
   `shared/lib/theme` module (System/Light/Dark, `prefers-color-scheme`
   listener, `.dark` on `<html>`, value in `localStorage` as a per-device
   convenience). Remove Manrope, lime, ink, pastel backdrop, `app-backdrop`.
2. **Shell.** Top bar with the parts above, bottom tab bar ≤ 720 px, width
   tiers, Needs you panel (waiting proposals, not logged by issue; "to
   confirm" was removed with plan 0005; no parallel-tickets block until plan 0002 phase 3). Settings moves
   behind a gear in the top bar. New Tick dial logo.
   Fixes Codex's open zoom-overflow finding by replacing that header.
3. **Day.** Header (arrows, title, totals), horizontal timeline restyle,
   vertical timeline, view switch in the zoom bar (stored like the zoom),
   floating details card / bottom sheet, Prompts lane. Small Rust change:
   `AgentBlock.promptTimesMs` from the stored `prompt_times` (no text).
   Highlight mode (`?issue=`) dims other blocks.
4. **Week.** Header with ← / → and "This week" (route param exists), totals,
   day cards with a sparkline, timesheet, `?issue=` row highlight. Small Rust
   change: `WeekDay.hourlyActiveMs` (24 values) for the sparkline.
5. **Log queue and Apps & rules.** Dense table, details pane, bulk approve
   (calls the existing approve command per id), filters; two-column rules
   page with private apps (time from `recent_apps`) and the rules table. Small
   Rust change: a `rule_matches` command (`RuleMatch { id, matchedMs }`), the time each rule named in the last 7 days; `Rule` stays unchanged for the store and MCP.
6. **Search palette.** New Rust command `search(query, days)` over stored
   data: issues (minutes per day and logged/proposed/detected state, from the
   week data), window titles and hosts, agent sessions (title, branch).
   Dates are parsed in the UI. Results open Day or Week with `?issue=`.
   Never searches prompt text; MCP does not get this command.
7. **Popover, Settings, app icon and loading states.** Restyle the
   menu-bar popover and Settings; generate the Tick dial icon set (1024 px
   master, `pnpm tauri icon`), tray icon in the same language; skeletons
   instead of loading text on every page (Łukasz, 2026-10-15).

Then update `ARCHITECTURE.md` (UI section) and the spec screenshots links.

## Review notes (Codex 016, adopted)

- Search (6): a bounded read-only query path, not seven `week_summary` builds
  per keystroke: limits on days (90), results and query length; UI debounce
  and stale-response guard; loading, empty, error and truncated states.
  Logged, proposed and detected minutes stay separate (they overlap). The
  backend applies the current privacy rules before it returns a title or
  snippet. Dates use local calendar days (DST-safe). Tests: private app and
  session titles, a title made private after capture, SQL wildcard input,
  zero results, a stale response after a newer query. No FTS index until a
  measured need.
- Prompt times (3): sorted, deduplicated, clipped to the half-open day and
  block range; the count derives from the same list. Tests: one timestamp in
  merged spans, a prompt exactly at day end, a session across midnight,
  zoom invariance. No phone provenance.
- Apps & rules (5): the private-app time is the rolling 14 days from
  `recent_apps`, labelled so; a private app hides details but does not yet
  exclude worklog minutes (plan 0002).
- Bulk approve (5): per-id results; keep successes, report and retry only
  the failed ids; only selected, visible, waiting rows; no double submit;
  refresh before showing success; approved stays distinct from logged.
- States (all): focus, disabled, selected and error states in both themes;
  contrast of orange, green and red on real light and dark surfaces, always
  with a text label; System mode with an OS change, a saved choice, blocked
  storage and the popover window.
- Overlays (2, 3, 6): portaled panels (Needs you, details sheet, drawer,
  palette) check against the viewport, not the container; keyboard focus
  return, Escape, scrolling. Test the tier edges 719/720/721 and
  959/960/961 plus 390/780/1440; the selected tab survives a tier change.
  The zoom-overflow finding closes only after a rendered check.
- `ARCHITECTURE.md` changes with each phase, not only at the end.

## Out of scope

- Prompt text on hover (separate small task decided in plan 0002).
- "From your phone" prompt marks: no data source today.
- Parallel tickets split in the Needs you panel (plan 0002 phase 3).

## Answers from Łukasz (2026-10-15)

1. Settings: a gear button in the top bar, next to the theme radio.
2. Keep the ≤ 720 px phone layout: a web or phone view to review worklogs
   may come later.
3. Add the backend data for week-card sparklines (hourly active time per
   day) and rule matches (time each rule named in the last 7 days).
4. Search covers the last 90 days.
