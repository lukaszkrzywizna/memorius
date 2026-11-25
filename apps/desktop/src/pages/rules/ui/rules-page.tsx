import type { AppUsage } from "@/shared/api/generated/AppUsage"
import type { Rule } from "@/shared/api/generated/Rule"
import type { RuleKind } from "@/shared/api/generated/RuleKind"
import { appNames } from "@/pages/rules/lib/rule-text"
import { IssueRulesSection } from "@/pages/rules/ui/issue-rules-section"
import { PrivateAppsSection } from "@/pages/rules/ui/private-apps-section"
import { SettingsSection } from "@/shared/ui/settings-section"
import { LoadingRegion, Skeleton } from "@/shared/ui/skeleton"

type RulesPageProps = {
  rules: readonly Rule[]
  apps: readonly AppUsage[]
  /** Time each rule named in the last 7 days, by rule id. */
  matches: ReadonlyMap<number, number> | undefined
  /** Rules or recent apps are still loading. */
  loading: boolean
  onAdd: (rule: RuleKind) => void
  onRemove: (id: number) => void
}

export function RulesPage({
  rules,
  apps,
  matches,
  loading,
  onAdd,
  onRemove,
}: RulesPageProps) {
  const appName = appNames(apps.map((usage) => usage.app))
  return (
    <div className="flex max-w-[1080px] flex-col gap-1.5 px-1 pt-1.5 pb-6">
      <header className="flex flex-col gap-1.5 pb-3">
        <h1 className="m-0 text-[34px] leading-none font-light tracking-tight text-t1">
          Apps &amp; rules
        </h1>
        <p className="m-0 text-t4">
          What Memorius records, and which issue it names.
        </p>
      </header>
      {loading ? (
        <LoadingRegion label="Loading rules" className="flex flex-col gap-1.5">
          {["Private apps", "Issue rules"].map((title) => (
            <SettingsSection
              key={title}
              id={`${title}-loading`}
              title={title}
              description=""
            >
              <div className="glass flex flex-col gap-3 rounded-2xl px-3.5 py-3">
                {[0, 1, 2].map((row) => (
                  <div key={row} className="flex items-center gap-3">
                    <Skeleton className="size-4.5 rounded-[5px]" />
                    <Skeleton className="h-3.5 flex-1" />
                    <Skeleton className="h-3.5 w-20" />
                  </div>
                ))}
              </div>
            </SettingsSection>
          ))}
        </LoadingRegion>
      ) : (
        <>
          <PrivateAppsSection
            rules={rules}
            apps={apps}
            appName={appName}
            onAdd={onAdd}
            onRemove={onRemove}
          />
          <IssueRulesSection
            rules={rules}
            apps={apps}
            matches={matches}
            appName={appName}
            onAdd={onAdd}
            onRemove={onRemove}
          />
        </>
      )}
    </div>
  )
}
