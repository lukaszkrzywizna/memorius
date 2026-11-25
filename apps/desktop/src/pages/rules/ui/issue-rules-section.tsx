import { XIcon } from "lucide-react"
import type { AppUsage } from "@/shared/api/generated/AppUsage"
import type { Rule } from "@/shared/api/generated/Rule"
import type { RuleKind } from "@/shared/api/generated/RuleKind"
import { ruleCells } from "@/pages/rules/lib/rule-text"
import { RULE_COLUMNS } from "@/pages/rules/lib/rule-columns"
import { IssueRuleForm } from "@/pages/rules/ui/issue-rule-form"
import { SettingsSection } from "@/shared/ui/settings-section"
import { formatDuration } from "@/shared/lib/duration"

type IssueRulesSectionProps = {
  rules: readonly Rule[]
  apps: readonly AppUsage[]
  /** Time each rule named in the last 7 days, by rule id; missing until loaded. */
  matches: ReadonlyMap<number, number> | undefined
  appName: (appId: string) => string
  onAdd: (rule: RuleKind) => void
  onRemove: (id: number) => void
}

export function IssueRulesSection({
  rules,
  apps,
  matches,
  appName,
  onAdd,
  onRemove,
}: IssueRulesSectionProps) {
  return (
    <SettingsSection
      id="issue-rules-heading"
      title="Issue rules"
      description="Name the issue for work without an issue key: a website, a meeting, an agent session or an app. A match counts as high confidence."
    >
      <div
        role="table"
        aria-label="Issue rules"
        className="glass overflow-hidden rounded-2xl text-[13px]"
      >
        <div
          role="row"
          className={`${RULE_COLUMNS} h-8 border-b border-border text-xs text-t4 max-[720px]:hidden`}
        >
          <span role="columnheader">Match</span>
          <span role="columnheader">Pattern</span>
          <span role="columnheader">Issue</span>
          <span role="columnheader">Matches · 7 days</span>
          <span role="columnheader" aria-label="Remove" />
        </div>
        {rules.map((rule) => {
          if (rule.rule.kind === "privateApp") return null
          const cells = ruleCells(rule.rule, appName)
          const matched = matches?.get(rule.id)
          return (
            <div
              key={rule.id}
              role="row"
              className={`${RULE_COLUMNS} min-h-10.5 border-b border-border max-[720px]:py-2`}
            >
              <span
                role="cell"
                className="text-t3 max-[720px]:[grid-area:kind]"
              >
                {cells.match}
              </span>
              <span
                role="cell"
                className="truncate text-t1 max-[720px]:[grid-area:pat]"
                title={cells.pattern}
              >
                {cells.pattern}
              </span>
              <span role="cell" className="max-[720px]:[grid-area:issue]">
                <span className="inline-flex h-5.5 items-center rounded-full border border-input px-2.5 text-xs text-t1">
                  {cells.issue}
                </span>
              </span>
              <span role="cell" className="text-t4 max-[720px]:hidden">
                {matched === undefined
                  ? "—"
                  : matched === 0
                    ? "none"
                    : formatDuration(matched)}
              </span>
              <span role="cell" className="max-[720px]:[grid-area:x]">
                <button
                  type="button"
                  aria-label={`Remove rule ${cells.pattern}`}
                  title="Remove"
                  onClick={() => onRemove(rule.id)}
                  className="grid size-7 place-items-center rounded-full text-t3 outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
                >
                  <XIcon aria-hidden="true" className="size-3" />
                </button>
              </span>
            </div>
          )
        })}
        <IssueRuleForm apps={apps} onAdd={onAdd} />
      </div>
    </SettingsSection>
  )
}
