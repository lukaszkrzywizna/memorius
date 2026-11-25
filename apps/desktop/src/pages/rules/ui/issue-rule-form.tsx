import { useId, useState } from "react"
import type { AppUsage } from "@/shared/api/generated/AppUsage"
import type { RuleKind } from "@/shared/api/generated/RuleKind"
import { isIssueKey } from "@/entities/issue"
import { RULE_COLUMNS } from "@/pages/rules/lib/rule-columns"
import { type IssueRuleKind, issueRule } from "@/pages/rules/lib/rule-text"
import { Button } from "@/shared/ui/button"
import { Input } from "@/shared/ui/input"
import { NativeSelect, NativeSelectOption } from "@/shared/ui/native-select"

type Draft = { kind: IssueRuleKind; pattern: string; issue: string }

const EMPTY: Draft = { kind: "hostIssue", pattern: "", issue: "" }

const PATTERN_LABEL: Record<IssueRuleKind, string> = {
  hostIssue: "Website host",
  titleIssue: "Window or session title contains",
  appIssue: "App",
}

export function IssueRuleForm({
  apps,
  onAdd,
}: {
  apps: readonly AppUsage[]
  onAdd: (rule: RuleKind) => void
}) {
  const id = useId()
  const [draft, setDraft] = useState(EMPTY)
  const valid = draft.pattern.trim() !== "" && isIssueKey(draft.issue)

  return (
    <form
      aria-label="Add a rule"
      className={`${RULE_COLUMNS} h-12 max-[720px]:h-auto max-[720px]:grid-cols-[minmax(0,1fr)_110px] max-[720px]:py-2.5 max-[720px]:[grid-template-areas:'kind_issue'_'pat_pat'_'btn_btn']`}
      onSubmit={(event) => {
        event.preventDefault()
        if (!valid) return
        onAdd(issueRule(draft.kind, draft.pattern.trim(), draft.issue.trim()))
        setDraft(EMPTY)
      }}
    >
      <NativeSelect
        aria-label="Match"
        className="max-[720px]:[grid-area:kind]"
        value={draft.kind}
        onChange={(event) =>
          setDraft({ ...draft, kind: toKind(event.target.value), pattern: "" })
        }
      >
        <NativeSelectOption value="hostIssue">Website</NativeSelectOption>
        <NativeSelectOption value="titleIssue">
          Title contains
        </NativeSelectOption>
        <NativeSelectOption value="appIssue">App</NativeSelectOption>
      </NativeSelect>
      {draft.kind === "appIssue" ? (
        <NativeSelect
          id={`${id}-pattern`}
          aria-label={PATTERN_LABEL[draft.kind]}
          className="w-full max-[720px]:[grid-area:pat]"
          value={draft.pattern}
          onChange={(event) =>
            setDraft({ ...draft, pattern: event.target.value })
          }
        >
          <NativeSelectOption value="">Choose an app</NativeSelectOption>
          {apps.map((usage) => (
            <NativeSelectOption key={usage.app.id} value={usage.app.id}>
              {usage.app.name}
            </NativeSelectOption>
          ))}
        </NativeSelect>
      ) : (
        <Input
          id={`${id}-pattern`}
          aria-label={PATTERN_LABEL[draft.kind]}
          className="max-[720px]:[grid-area:pat]"
          value={draft.pattern}
          placeholder={
            draft.kind === "hostIssue" ? "orders.example.com" : "Daily stand-up"
          }
          onChange={(event) =>
            setDraft({ ...draft, pattern: event.target.value })
          }
        />
      )}
      <Input
        aria-label="Issue"
        className="max-[720px]:[grid-area:issue]"
        value={draft.issue}
        placeholder="SEL-54"
        onChange={(event) => setDraft({ ...draft, issue: event.target.value })}
      />
      <Button
        type="submit"
        disabled={!valid}
        className="max-[720px]:[grid-area:btn]"
      >
        Add rule
      </Button>
    </form>
  )
}

function toKind(value: string): IssueRuleKind {
  return value === "appIssue" || value === "titleIssue" ? value : "hostIssue"
}
