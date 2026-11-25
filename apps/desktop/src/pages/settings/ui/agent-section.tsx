import type { ReactNode } from "react"
import type { AgentCheck } from "@/shared/api/generated/AgentCheck"
import type { AgentKind } from "@/shared/api/generated/AgentKind"
import type { AgentSettings } from "@/shared/api/generated/AgentSettings"
import type { RunRange } from "@/shared/api/generated/RunRange"
import { AGENT_NAMES, RANGE_LABELS } from "@/entities/agent"
import { Button } from "@/shared/ui/button"
import { Input } from "@/shared/ui/input"
import { NativeSelect } from "@/shared/ui/native-select"
import { SettingsSection } from "@/shared/ui/settings-section"
import { Textarea } from "@/shared/ui/textarea"

type AgentSectionProps = {
  settings: AgentSettings | undefined
  check: AgentCheck | undefined
  /** The proposals prompt the agent gets now, with the user's instructions. */
  prompt: string | undefined
  onChange: (settings: AgentSettings) => void
  /** The memorius command and MCP server for the user's own sessions. */
  commandLine: {
    note: string | undefined
    onInstall: () => void
    onAdd: (agent: "claudeCode" | "codex") => void
  }
}

/** Which agent Memorius starts, what it is told, and the daily run. */
export function AgentSection({
  settings,
  check,
  prompt,
  onChange,
  commandLine,
}: AgentSectionProps) {
  return (
    <SettingsSection
      id="agent-heading"
      title="Agent"
      description="Memorius starts your coding agent's own CLI, so your subscription pays. The agent reads your activity, decides which issue each stretch of work belongs to and proposes worklogs; you approve them in the Log queue."
    >
      {settings && (
        <div className="glass flex flex-col gap-4 rounded-2xl px-4 py-3.5 text-[13px]">
          <Row label="Agent">
            <NativeSelect
              aria-label="Agent"
              value={settings.agent}
              onChange={(event) =>
                onChange({
                  ...settings,
                  agent: event.target.value as AgentKind,
                })
              }
            >
              {Object.entries(AGENT_NAMES).map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </NativeSelect>
          </Row>
          {settings.agent === "custom" ? (
            <Row label="Command" hint="{prompt} is replaced by the prompt.">
              <TextSetting
                label="Command"
                value={settings.customCommand}
                placeholder="my-agent --prompt {prompt}"
                onSave={(customCommand) =>
                  onChange({ ...settings, customCommand })
                }
              />
            </Row>
          ) : (
            <Row
              label={`${AGENT_NAMES[settings.agent]} CLI`}
              hint="The program Memorius starts."
            >
              <Found path={check?.programPath} loading={!check} />
            </Row>
          )}
          <Row
            label="Command line"
            hint="For your own Claude Code or Codex sessions: Install command links memorius in ~/.local/bin; Add to … adds a memorius MCP server to that agent's user config and copies the memorius-worklogs skill. Memorius's own runs need none of this."
          >
            <Found path={check?.memoriusPath} loading={!check} />
            <span className="flex flex-wrap gap-2">
              <Button
                size="sm"
                variant="outline"
                onClick={commandLine.onInstall}
              >
                Install command
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={() => commandLine.onAdd("claudeCode")}
              >
                Add to Claude Code
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={() => commandLine.onAdd("codex")}
              >
                Add to Codex
              </Button>
            </span>
            {commandLine.note && (
              <span role="status" className="text-xs text-t3">
                {commandLine.note}
              </span>
            )}
          </Row>
          <Row label="Every day">
            <label className="flex items-center gap-2 text-t2">
              <input
                type="checkbox"
                checked={settings.daily}
                onChange={(event) =>
                  onChange({ ...settings, daily: event.target.checked })
                }
              />
              Prepare proposals at
            </label>
            <Input
              type="time"
              aria-label="Daily run time"
              className="w-28"
              value={settings.dailyTime}
              disabled={!settings.daily}
              onChange={(event) =>
                onChange({ ...settings, dailyTime: event.target.value })
              }
            />
          </Row>
          <Row label="Range" hint="How far back a run looks by default.">
            <NativeSelect
              aria-label="Range"
              value={settings.defaultRange}
              onChange={(event) =>
                onChange({
                  ...settings,
                  defaultRange: event.target.value as RunRange,
                })
              }
            >
              {Object.entries(RANGE_LABELS).map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </NativeSelect>
          </Row>
          <Row
            label="Issue lookup"
            hint="Read-only tracker tools for checking issues while proposing, as server/tool, comma separated. Empty: Memorius data only."
          >
            <TextSetting
              label="Issue lookup"
              value={settings.lookupTools}
              onSave={(lookupTools) => onChange({ ...settings, lookupTools })}
            />
          </Row>
          <Row
            label="Claude Code data"
            hint="Where Claude Code keeps its sessions, which Memorius reads for agent time. Empty: CLAUDE_CONFIG_DIR or ~/.claude."
          >
            <TextSetting
              label="Claude Code data folder"
              value={settings.claudeHome}
              placeholder="~/.claude"
              onSave={(claudeHome) => onChange({ ...settings, claudeHome })}
            />
          </Row>
          <Row
            label="Codex data"
            hint="Where Codex keeps its sessions. Empty: CODEX_HOME or ~/.codex."
          >
            <TextSetting
              label="Codex data folder"
              value={settings.codexHome}
              placeholder="~/.codex"
              onSave={(codexHome) => onChange({ ...settings, codexHome })}
            />
          </Row>
          <Row
            label="Instructions"
            hint="Added to the agent's prompt when it proposes."
          >
            <Textarea
              key={settings.instructions}
              aria-label="Instructions"
              placeholder="For example: round to 15 minutes; log Memorius work under SEL-21; work type Development."
              defaultValue={settings.instructions}
              onBlur={(event) =>
                onChange({ ...settings, instructions: event.target.value })
              }
            />
          </Row>
          {prompt && (
            <details className="border-t border-border pt-3 text-t3">
              <summary className="cursor-pointer text-t2">
                What the agent gets
              </summary>
              <pre className="m-0 mt-2 rounded-[10px] bg-muted px-3 py-2.5 font-mono text-xs leading-normal whitespace-pre-wrap text-t2">
                {prompt}
              </pre>
            </details>
          )}
        </div>
      )}
    </SettingsSection>
  )
}

function Row({
  label,
  hint,
  children,
}: {
  label: string
  hint?: string
  children: ReactNode
}) {
  return (
    <div className="grid grid-cols-[140px_minmax(0,1fr)] items-start gap-3 max-[720px]:grid-cols-1 max-[720px]:gap-1.5">
      <span className="pt-1.5 text-t4">{label}</span>
      <span className="flex min-w-0 flex-col gap-1">
        <span className="flex flex-wrap items-center gap-2">{children}</span>
        {hint && <span className="text-xs text-t4">{hint}</span>}
      </span>
    </div>
  )
}

function TextSetting({
  label,
  value,
  placeholder,
  onSave,
}: {
  label: string
  value: string
  placeholder?: string
  onSave: (value: string) => void
}) {
  return (
    <Input
      key={value}
      aria-label={label}
      className="max-w-96 font-mono"
      placeholder={placeholder}
      defaultValue={value}
      onBlur={(event) => onSave(event.target.value.trim())}
    />
  )
}

function Found({
  path,
  loading,
}: {
  path: string | null | undefined
  loading: boolean
}) {
  return (
    <span className="flex min-h-8 min-w-0 items-center gap-2 text-t2">
      <span
        aria-hidden="true"
        className={`size-1.5 shrink-0 ${
          loading ? "bg-t5" : path ? "bg-operational" : "bg-critical"
        }`}
      />
      {loading ? (
        "Checking…"
      ) : path ? (
        <>
          Found
          <span className="truncate font-mono text-xs text-t4" title={path}>
            {path}
          </span>
        </>
      ) : (
        "Not found in your shell's PATH"
      )}
    </span>
  )
}
