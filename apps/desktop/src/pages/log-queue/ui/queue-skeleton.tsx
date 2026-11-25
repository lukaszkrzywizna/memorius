import { LoadingRegion, Skeleton } from "@/shared/ui/skeleton"

const ROWS = 7

/** Proposal rows while the queue loads. */
export function QueueSkeleton() {
  return (
    <LoadingRegion
      label="Loading proposals"
      className="glass rounded-[18px] py-1"
    >
      <div className="flex h-8.5 items-center gap-3 border-b border-border px-3.5">
        <Skeleton className="h-3 w-24" />
      </div>
      {Array.from({ length: ROWS }, (_, index) => (
        <div
          key={index}
          className="flex h-11 items-center gap-3 border-b border-border px-3.5"
        >
          <Skeleton className="size-3.5 rounded-sm" />
          <Skeleton className="h-3.5 w-14" />
          <Skeleton className="h-3 w-28 max-[960px]:hidden" />
          <Skeleton className="h-3.5 flex-1" />
          <Skeleton className="h-3.5 w-12" />
          <Skeleton className="h-7 w-20 rounded-full" />
        </div>
      ))}
    </LoadingRegion>
  )
}
