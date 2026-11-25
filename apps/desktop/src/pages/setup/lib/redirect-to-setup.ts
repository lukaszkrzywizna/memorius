import { redirect } from "@tanstack/react-router"
import { readSetupStatus } from "@/shared/api/memorius-commands"

/** Opens setup until the user has finished it once. */
export async function redirectToSetupWhenNew() {
  if (!(await readSetupStatus()).done) {
    redirect({ to: "/setup", throw: true })
  }
}
