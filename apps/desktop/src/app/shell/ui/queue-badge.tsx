/** The number of waiting proposals next to the Log queue tab. */
export function QueueBadge({ count }: { count: number }) {
  if (count === 0) return null
  return (
    <span
      aria-hidden="true"
      className="grid h-4 min-w-4 place-items-center rounded-full bg-attention px-1 text-[10px] font-medium text-attention-foreground"
    >
      {count}
    </span>
  )
}
