import { invoke } from "@tauri-apps/api/core"
import { queryOptions } from "@tanstack/react-query"
import type { AgentCheck } from "@/shared/api/generated/AgentCheck"
import type { AgentRun } from "@/shared/api/generated/AgentRun"
import type { AgentSettings } from "@/shared/api/generated/AgentSettings"
import type { AppUsage } from "@/shared/api/generated/AppUsage"
import type { CommitDetails } from "@/shared/api/generated/CommitDetails"
import type { DaySummary } from "@/shared/api/generated/DaySummary"
import type { GitScan } from "@/shared/api/generated/GitScan"
import type { GitSettings } from "@/shared/api/generated/GitSettings"
import type { GitStatus } from "@/shared/api/generated/GitStatus"
import type { TimeRules } from "@/shared/api/generated/TimeRules"
import type { UpdateStatus } from "@/shared/api/generated/UpdateStatus"
import type { AgentAuth } from "@/shared/api/generated/AgentAuth"
import type { AgentKind } from "@/shared/api/generated/AgentKind"
import type { SetupStatus } from "@/shared/api/generated/SetupStatus"
import type { SystemPane } from "@/shared/api/generated/SystemPane"
import type { DayTimeline } from "@/shared/api/generated/DayTimeline"
import type { LogResult } from "@/shared/api/generated/LogResult"
import type { Proposal } from "@/shared/api/generated/Proposal"
import type { ProposalEdit } from "@/shared/api/generated/ProposalEdit"
import type { QueueItem } from "@/shared/api/generated/QueueItem"
import type { RecorderStatus } from "@/shared/api/generated/RecorderStatus"
import type { ReminderSettings } from "@/shared/api/generated/ReminderSettings"
import type { Rule } from "@/shared/api/generated/Rule"
import type { RuleKind } from "@/shared/api/generated/RuleKind"
import type { RuleMatch } from "@/shared/api/generated/RuleMatch"
import type { RunRange } from "@/shared/api/generated/RunRange"
import type { SearchResults } from "@/shared/api/generated/SearchResults"
import type { SessionRef } from "@/shared/api/generated/SessionRef"
import type { SyncState } from "@/shared/api/generated/SyncState"
import type { TrackerStatus } from "@/shared/api/generated/TrackerStatus"
import type { WeekSummary } from "@/shared/api/generated/WeekSummary"

/** Root key of every Memorius query; activity events invalidate it. */
export const memoriusQueryKey = ["memorius"] as const

/** One day in slots of `zoomMinutes` (1, 5, 10, 15, 30 or 60). */
export function dayTimelineQuery(date: string, zoomMinutes: number) {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "day-timeline", date, zoomMinutes],
    queryFn: () => invoke<DayTimeline>("day_timeline", { date, zoomMinutes }),
  })
}

/** The current window changes every second, so the status is polled while a window is open. */
export function recorderStatusQuery() {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "recorder-status"],
    queryFn: () => invoke<RecorderStatus>("recorder_status"),
    refetchInterval: 2_000,
  })
}

export function pauseRecording(minutes: number) {
  return invoke<null>("pause_recording", { minutes })
}

export function resumeRecording() {
  return invoke<null>("resume_recording")
}

export function requestTitleAccess() {
  return invoke<null>("request_title_access")
}

export function showMainWindow() {
  return invoke<null>("show_main_window")
}

export function restartApp() {
  return invoke<null>("restart_app")
}

export function quitApp() {
  return invoke<null>("quit_app")
}

export function rulesQuery() {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "rules"],
    queryFn: () => invoke<Rule[]>("rules"),
  })
}

/** Time each rule named in the last 7 days; it builds 7 days, so it loads apart. */
export function ruleMatchesQuery() {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "rule-matches"],
    queryFn: () => invoke<RuleMatch[]>("rule_matches"),
  })
}

/** Stored windows, sessions and proposals of the last 90 days that match. */
export function searchQuery(query: string) {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "search", query],
    queryFn: () => invoke<SearchResults>("search", { query }),
  })
}

/** A prompt's text, read from its session file only while the user hovers it. */
export function promptTextQuery(sessions: SessionRef[], atMs: number) {
  return queryOptions({
    queryKey: [
      ...memoriusQueryKey,
      "prompt-text",
      sessions.map((session) => `${session.source}:${session.sessionId}`),
      atMs,
    ],
    queryFn: () => invoke<string | null>("prompt_text", { sessions, atMs }),
    staleTime: Number.POSITIVE_INFINITY,
  })
}

export function recentAppsQuery() {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "recent-apps"],
    queryFn: () => invoke<AppUsage[]>("recent_apps"),
  })
}

export function addRule(rule: RuleKind) {
  return invoke<null>("add_rule", { rule })
}

export function removeRule(id: number) {
  return invoke<null>("remove_rule", { id })
}

/** Agents add proposals from outside the app, so the queue is polled while open. */
export function logQueueQuery() {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "log-queue"],
    queryFn: () => invoke<QueueItem[]>("log_queue"),
    refetchInterval: 10_000,
  })
}

export function weekSummaryQuery(week: string) {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "week", week],
    queryFn: () => invoke<WeekSummary>("week_summary", { week }),
  })
}

/** Approves the revision the user saw; a newer one is refused. */
export function approveProposal({
  id,
  revision,
  edit,
}: {
  id: number
  revision: number
  edit: ProposalEdit | null
}) {
  return invoke<Proposal>("approve_proposal", { id, revision, edit })
}

export type SessionsPrivateRequest = {
  sessions: SessionRef[]
  /** `false` counts the sessions again (undo). */
  private: boolean
}

/** Marks sessions private: their prompts and agent time never count. */
export function setSessionsPrivate({
  sessions,
  private: hidden,
}: SessionsPrivateRequest) {
  return invoke<null>("set_sessions_private", { sessions, private: hidden })
}

export function rejectProposal(id: number) {
  return invoke<Proposal>("reject_proposal", { id })
}

export function undoProposal(id: number) {
  return invoke<Proposal>("undo_proposal", { id })
}

export function agentSettingsQuery() {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "agent-settings"],
    queryFn: () => invoke<AgentSettings>("agent_settings"),
  })
}

export function saveAgentSettings(settings: AgentSettings) {
  return invoke<null>("save_agent_settings", { settings })
}

/** Where the agent CLI and `memorius` were found in the login shell's PATH. */
export function agentCheckQuery() {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "agent-check"],
    queryFn: () => invoke<AgentCheck>("agent_check"),
  })
}

/** The proposals prompt a run would get now, shown in Settings. */
export function agentPromptQuery() {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "agent-prompt"],
    queryFn: () => invoke<string>("agent_prompt"),
  })
}

/** The newest runs first; polled while one is running, so its end shows. */
export function agentRunsQuery() {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "agent-runs"],
    queryFn: () => invoke<AgentRun[]>("agent_runs"),
    refetchInterval: (query) =>
      query.state.data?.[0]?.state.kind === "running" ? 2_000 : false,
  })
}

export function startAgentRun(range: RunRange) {
  return invoke<null>("start_agent_run", { range })
}

export function stopAgentRun() {
  return invoke<null>("stop_agent_run")
}

export function trackerStatusQuery() {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "tracker-status"],
    queryFn: () => invoke<TrackerStatus>("tracker_status"),
  })
}

/** Checks the address and token against YouTrack, saves them and syncs; returns the user's name. */
export function connectTracker({
  url,
  token,
}: {
  url: string
  token: string | null
}) {
  return invoke<string>("connect_tracker", { url, token })
}

export function disconnectTracker() {
  return invoke<null>("disconnect_tracker")
}

export function syncTracker() {
  return invoke<SyncState>("sync_tracker")
}

export function reminderSettingsQuery() {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "reminder-settings"],
    queryFn: () => invoke<ReminderSettings>("reminder_settings"),
  })
}

export function saveReminderSettings(settings: ReminderSettings) {
  return invoke<null>("save_reminder_settings", { settings })
}

/** Shows the reminder now, to check that notifications reach the user. */
export function testReminder() {
  return invoke<null>("test_reminder")
}

/** Logs the approved proposals in YouTrack with the user's token. */
/** Logs the approved proposals in YouTrack: all, or only `ids`. */
export function logApproved(ids?: number[]) {
  return invoke<LogResult>("log_approved", { ids: ids ?? null })
}

/** The tracker's work types from the last sync; empty before one. */
export function trackerWorkTypesQuery() {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "tracker-work-types"],
    queryFn: () => invoke<string[]>("tracker_work_types"),
  })
}

/** Totals of the given days (`YYYY-MM-DD`), for the Log queue's day rows. */
export function daySummariesQuery(dates: readonly string[]) {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "day-summaries", ...dates],
    queryFn: () => invoke<DaySummary[]>("day_summaries", { dates }),
    enabled: dates.length > 0,
  })
}

export function gitStatusQuery() {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "git-status"],
    queryFn: () => invoke<GitStatus>("git_status"),
  })
}

/** Saves the repositories folder and reads the commits again. */
export function saveGitSettings(settings: GitSettings) {
  return invoke<GitScan>("save_git_settings", { settings })
}

export function timeRulesQuery() {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "time-rules"],
    queryFn: () => invoke<TimeRules>("time_rules"),
  })
}

export function saveTimeRules(rules: TimeRules) {
  return invoke<TimeRules>("save_time_rules", { rules })
}

/** The installed version and a newer release on GitHub, if any. */
export function updateStatusQuery() {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "update-status"],
    queryFn: () => invoke<UpdateStatus>("check_update"),
    retry: false,
  })
}

/** Installs the newer release and restarts Memorius. */
export function installUpdate() {
  return invoke<null>("install_update")
}

/** A commit's message, changed files and GitHub link, read from git. */
export function commitDetailsQuery(hash: string) {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "commit-details", hash],
    queryFn: () => invoke<CommitDetails>("commit_details", { hash }),
    staleTime: Number.POSITIVE_INFINITY,
  })
}

/** Opens an https link in the default browser. */
export function openLink(url: string) {
  return invoke<null>("open_link", { url })
}

/** What setup still needs: start at login and notifications. */
export function readSetupStatus() {
  return invoke<SetupStatus>("setup_status")
}

export function setupStatusQuery() {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "setup-status"],
    queryFn: readSetupStatus,
  })
}

export function finishSetup() {
  return invoke<null>("finish_setup")
}

/** The user saw the test notification arrive. */
export function confirmNotifications() {
  return invoke<null>("confirm_notifications")
}

export function startAtLogin() {
  return invoke<null>("start_at_login")
}

export function openSystemPane(pane: SystemPane) {
  return invoke<null>("open_system_pane", { pane })
}

/** Whether the agent from Settings is installed and signed in. */
export function agentAuthQuery() {
  return queryOptions({
    queryKey: [...memoriusQueryKey, "agent-auth"],
    queryFn: () => invoke<AgentAuth>("agent_auth"),
    staleTime: 10 * 60_000,
  })
}

/** Links the CLI inside the app as the `memorius` command; says what is left. */
export function installCli() {
  return invoke<string>("install_cli")
}

/** Registers Memorius's MCP server and skill in Claude Code or Codex. */
export function addToAgent(agent: AgentKind) {
  return invoke<null>("add_to_agent", { agent })
}

/** Opens a terminal with the agent's own sign-in. */
export function agentSignIn() {
  return invoke<null>("agent_sign_in")
}
