import { TriangleAlertIcon } from "lucide-react"
import { Button } from "@/shared/ui/button"

type AttentionBannerProps = {
  /** Critical when Memorius records less; attention when only a feature stops. */
  tone: "critical" | "attention"
  title: string
  text: string
  action: string
  onAction: () => void
}

/** Something that stopped while Memorius ran, and the one button that fixes it. */
export function AttentionBanner({
  tone,
  title,
  text,
  action,
  onAction,
}: AttentionBannerProps) {
  return (
    <div
      role="alert"
      className={`mb-3 flex flex-wrap items-center gap-3 rounded-2xl border px-3.5 py-2.5 text-[13px] text-t1 ${
        tone === "critical"
          ? "border-critical/40 bg-critical/10"
          : "border-attention/50 bg-attention/12"
      }`}
    >
      <TriangleAlertIcon
        aria-hidden="true"
        className={`size-4.5 shrink-0 ${tone === "critical" ? "text-critical" : "text-attention"}`}
      />
      <span className="min-w-0 flex-1">
        <strong className="font-medium">{title}</strong> {text}
      </span>
      <Button
        size="sm"
        variant={tone === "critical" ? "default" : "outline"}
        onClick={onAction}
      >
        {action}
      </Button>
    </div>
  )
}
