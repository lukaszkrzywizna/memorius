import { useId, useState } from "react"
import { isIssueKey } from "@/entities/issue"
import type { Proposal } from "@/shared/api/generated/Proposal"
import type { ProposalEdit } from "@/shared/api/generated/ProposalEdit"
import { Button } from "@/shared/ui/button"
import { Input } from "@/shared/ui/input"
import { NativeSelect } from "@/shared/ui/native-select"
import { Textarea } from "@/shared/ui/textarea"

const labelStyle = "text-xs text-muted-foreground"

type ProposalEditFormProps = {
  proposal: Proposal
  /** The tracker's work types; empty before the first sync (free text). */
  workTypes: readonly string[]
  onApprove: (edit: ProposalEdit) => void
  onCancel: () => void
}

/** Change the values before approving; the agent logs exactly these. */
export function ProposalEditForm({
  proposal,
  workTypes,
  onApprove,
  onCancel,
}: ProposalEditFormProps) {
  const id = useId()
  const rangeMinutes = Math.floor((proposal.endMs - proposal.startMs) / 60_000)
  const [draft, setDraft] = useState({
    issue: proposal.issue,
    minutes: String(proposal.minutes),
    workType: proposal.workType ?? "",
    text: proposal.text,
  })
  const minutes = Number(draft.minutes)
  const valid =
    isIssueKey(draft.issue) &&
    Number.isInteger(minutes) &&
    minutes >= 1 &&
    minutes <= rangeMinutes &&
    draft.text.trim() !== "" &&
    (workTypes.length === 0 ||
      draft.workType === "" ||
      workTypes.includes(draft.workType))

  return (
    <form
      aria-label="Edit worklog"
      className="flex flex-col gap-3"
      onSubmit={(event) => {
        event.preventDefault()
        if (!valid) return
        onApprove({
          issue: draft.issue.trim(),
          minutes,
          workType: draft.workType.trim() === "" ? null : draft.workType.trim(),
          text: draft.text.trim(),
        })
      }}
    >
      <div className="flex flex-wrap gap-2">
        <div className="flex w-32 flex-col gap-1">
          <label htmlFor={`${id}-issue`} className={labelStyle}>
            Issue
          </label>
          <Input
            id={`${id}-issue`}
            className="bg-muted"
            value={draft.issue}
            onChange={(event) =>
              setDraft({ ...draft, issue: event.target.value })
            }
          />
        </div>
        <div className="flex w-28 flex-col gap-1">
          <label htmlFor={`${id}-minutes`} className={labelStyle}>
            Minutes (max {rangeMinutes})
          </label>
          <Input
            id={`${id}-minutes`}
            className="bg-muted"
            inputMode="numeric"
            value={draft.minutes}
            onChange={(event) =>
              setDraft({ ...draft, minutes: event.target.value })
            }
          />
        </div>
        <div className="flex min-w-40 flex-1 flex-col gap-1">
          <label htmlFor={`${id}-type`} className={labelStyle}>
            Work type
          </label>
          {workTypes.length > 0 ? (
            <NativeSelect
              id={`${id}-type`}
              className="w-full"
              value={draft.workType}
              onChange={(event) =>
                setDraft({ ...draft, workType: event.target.value })
              }
            >
              <option value="">No type</option>
              {!workTypes.includes(draft.workType) && draft.workType && (
                <option value={draft.workType}>
                  {draft.workType} (not in YouTrack)
                </option>
              )}
              {workTypes.map((type) => (
                <option key={type} value={type}>
                  {type}
                </option>
              ))}
            </NativeSelect>
          ) : (
            <Input
              id={`${id}-type`}
              className="bg-muted"
              value={draft.workType}
              placeholder="Development"
              onChange={(event) =>
                setDraft({ ...draft, workType: event.target.value })
              }
            />
          )}
        </div>
      </div>
      <div className="flex flex-col gap-1">
        <label htmlFor={`${id}-text`} className={labelStyle}>
          Description
        </label>
        <Textarea
          id={`${id}-text`}
          className="min-h-20 bg-muted"
          value={draft.text}
          onChange={(event) => setDraft({ ...draft, text: event.target.value })}
        />
      </div>
      <div className="flex gap-2">
        <Button type="submit" size="lg" disabled={!valid}>
          Approve with changes
        </Button>
        <Button type="button" size="lg" variant="ghost" onClick={onCancel}>
          Cancel
        </Button>
      </div>
    </form>
  )
}
