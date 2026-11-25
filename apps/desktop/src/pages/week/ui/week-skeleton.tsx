import { LoadingRegion, Skeleton } from "@/shared/ui/skeleton"

const DAYS = 5
const ROWS = 5

/** Day cards and timesheet rows while the week loads. */
export function WeekSkeleton() {
  return (
    <LoadingRegion label="Loading the week" className="flex flex-col gap-4">
      <div className="grid grid-cols-[repeat(auto-fit,minmax(150px,1fr))] gap-3">
        {Array.from({ length: DAYS }, (_, index) => (
          <div
            key={index}
            className="glass flex flex-col gap-3 rounded-[20px] px-4 py-3.5"
          >
            <Skeleton className="h-3.5 w-20" />
            <Skeleton className="h-7 w-24" />
            <Skeleton className="h-6 w-full" />
            <Skeleton className="h-2 w-full" />
          </div>
        ))}
      </div>
      <div className="glass flex flex-col gap-3.5 rounded-[22px] px-5 py-4">
        {Array.from({ length: ROWS }, (_, index) => (
          <div key={index} className="flex items-center gap-4">
            <Skeleton className="h-3.5 w-16" />
            {Array.from({ length: DAYS }, (_, cell) => (
              <Skeleton key={cell} className="h-6.5 flex-1 rounded-full" />
            ))}
            <Skeleton className="h-3.5 w-12" />
          </div>
        ))}
      </div>
    </LoadingRegion>
  )
}
