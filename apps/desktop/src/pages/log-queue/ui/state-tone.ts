import type { QueueState } from "@/pages/log-queue/lib/queue-rows"

/** One colour per proposal state, the same on filters, sections and rows. */
export const STATE_DOT: Record<QueueState, string> = {
  proposed: "bg-pending",
  approved: "bg-attention",
  logged: "bg-operational",
  rejected: "bg-t5",
}
