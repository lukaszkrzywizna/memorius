import { cn } from "cn"

/** A calm placeholder in the shape of content that is still loading. */
function Skeleton({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="skeleton"
      aria-hidden="true"
      className={cn(
        "animate-pulse rounded-lg bg-t1/8 motion-reduce:animate-none",
        className,
      )}
      {...props}
    />
  )
}

/** Names a group of skeletons once for screen readers. */
function LoadingRegion({
  label,
  className,
  children,
}: {
  label: string
  className?: string
  children: React.ReactNode
}) {
  return (
    <div role="status" aria-label={label} className={className}>
      <span className="sr-only">{label}</span>
      {children}
    </div>
  )
}

export { LoadingRegion, Skeleton }
