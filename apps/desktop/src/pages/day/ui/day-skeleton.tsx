import { LoadingRegion, Skeleton } from "@/shared/ui/skeleton"

/** Lanes with grey blocks where the day's activity will appear. */
const LANES = [
  {
    label: "Apps",
    top: 32,
    height: 80,
    blocks: [
      [0, 9],
      [10, 22],
      [33, 8],
      [42, 14],
      [57, 20],
    ],
  },
  {
    label: "Agents",
    top: 128,
    height: 38,
    blocks: [
      [4, 12],
      [26, 18],
      [52, 24],
    ],
  },
  {
    label: "Work",
    top: 232,
    height: 50,
    blocks: [
      [0, 8],
      [9, 23],
      [33, 9],
      [43, 33],
    ],
  },
] as const

export function DaySkeleton() {
  return (
    <LoadingRegion
      label="Loading the day"
      className="relative min-h-[420px] flex-1"
    >
      {LANES.map((lane) => (
        <span
          key={lane.label}
          aria-hidden="true"
          className="absolute left-6.5 text-t5 max-[720px]:left-0.5"
          style={{ top: lane.top + lane.height / 2 - 8 }}
        >
          {lane.label}
        </span>
      ))}
      <div className="absolute inset-[0_16px_64px_96px] overflow-hidden max-[720px]:inset-[0_0_56px_52px]">
        {LANES.map((lane) =>
          lane.blocks.map(([left, width]) => (
            <Skeleton
              key={`${lane.label}:${left}`}
              className="absolute rounded-[10px]"
              style={{
                top: lane.top,
                height: lane.height,
                left: `${left}%`,
                width: `${width}%`,
              }}
            />
          )),
        )}
      </div>
    </LoadingRegion>
  )
}
