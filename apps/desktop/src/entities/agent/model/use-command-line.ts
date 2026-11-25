import { useMutation, useQueryClient } from "@tanstack/react-query"
import {
  addToAgent,
  installCli,
  memoriusQueryKey,
} from "@/shared/api/memorius-commands"

/**
 * The command line buttons: link the memorius command, or register its MCP
 * server and skill with an agent. `note` tells how the last click went.
 */
export function useCommandLine() {
  const queryClient = useQueryClient()
  const install = useMutation({
    mutationFn: installCli,
    onSettled: () =>
      queryClient.invalidateQueries({ queryKey: memoriusQueryKey }),
  })
  const add = useMutation({ mutationFn: addToAgent })
  const added =
    add.isSuccess &&
    `Added to ${add.variables === "codex" ? "Codex" : "Claude Code"}.`
  return {
    note:
      [install.error, add.error].find(Boolean)?.toString() ??
      (install.data || added || undefined),
    onInstall: () => {
      add.reset()
      install.mutate()
    },
    onAdd: (agent: "claudeCode" | "codex") => {
      install.reset()
      add.mutate(agent)
    },
  }
}
