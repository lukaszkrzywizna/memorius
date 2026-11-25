import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { RulesPage } from "@/pages/rules/ui/rules-page"
import {
  addRule,
  memoriusQueryKey,
  recentAppsQuery,
  removeRule,
  ruleMatchesQuery,
  rulesQuery,
} from "@/shared/api/memorius-commands"

export function RulesRouteComponent() {
  const queryClient = useQueryClient()
  const rules = useQuery(rulesQuery())
  const apps = useQuery(recentAppsQuery())
  const matches = useQuery(ruleMatchesQuery()).data
  // A rule changes detected work on every day, so all views refresh.
  const refresh = () =>
    queryClient.invalidateQueries({ queryKey: memoriusQueryKey })
  const add = useMutation({ mutationFn: addRule, onSuccess: refresh })
  const remove = useMutation({ mutationFn: removeRule, onSuccess: refresh })

  return (
    <RulesPage
      rules={rules.data ?? []}
      apps={apps.data ?? []}
      loading={rules.isPending || apps.isPending}
      matches={
        matches && new Map(matches.map((match) => [match.id, match.matchedMs]))
      }
      onAdd={(rule) => add.mutate(rule)}
      onRemove={(id) => remove.mutate(id)}
    />
  )
}
