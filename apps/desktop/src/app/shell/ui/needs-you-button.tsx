import { useNavigate } from "@tanstack/react-router"
import { InboxIcon } from "lucide-react"
import { useState } from "react"
import { type NeedsYou, useNeedsYou } from "@/app/shell/model/needs-you"
import { NeedsYouPanel } from "@/app/shell/ui/needs-you-panel"
import {
  Popover,
  PopoverContent,
  PopoverTitle,
  PopoverTrigger,
} from "@/shared/ui/popover"

/** The top-bar icon with today's count; the panel opens on click, on every page. */
export function NeedsYouButton() {
  const needs = useNeedsYou()
  const navigate = useNavigate()
  return (
    <NeedsYouPopover
      needs={needs}
      onOpenQueue={() => void navigate({ to: "/queue" })}
    />
  )
}

type NeedsYouPopoverProps = {
  needs: NeedsYou
  onOpenQueue: () => void
  defaultOpen?: boolean
}

export function NeedsYouPopover({
  needs,
  onOpenQueue,
  defaultOpen = false,
}: NeedsYouPopoverProps) {
  const [open, setOpen] = useState(defaultOpen)
  const name =
    needs.count > 0 ? `Needs you, ${needs.count} items` : "Needs you, nothing"
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger
        aria-label={name}
        title="Needs you"
        className="glass relative grid size-10 place-items-center rounded-full text-t2 outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring data-popup-open:bg-selected"
      >
        <InboxIcon aria-hidden="true" className="size-4" />
        {needs.count > 0 && (
          <span
            aria-hidden="true"
            className="absolute -top-1 -right-1 grid h-[18px] min-w-[18px] place-items-center rounded-full bg-attention px-1 text-[11px] font-medium text-attention-foreground"
          >
            {needs.count}
          </span>
        )}
      </PopoverTrigger>
      <PopoverContent
        align="end"
        sideOffset={10}
        className="max-h-[min(640px,calc(100vh-96px))] w-[min(340px,calc(100vw-20px))] gap-3 overflow-y-auto warm-panel!"
      >
        <PopoverTitle className="text-lg">Needs you</PopoverTitle>
        <NeedsYouPanel
          needs={needs}
          onOpenQueue={() => {
            setOpen(false)
            onOpenQueue()
          }}
        />
      </PopoverContent>
    </Popover>
  )
}
