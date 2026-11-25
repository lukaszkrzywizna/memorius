const TICKS = Array.from({ length: 12 }, (_, index) => index * 30)

/** The Tick dial: a day of ticks with the logged part as one orange arc. */
export function Logo() {
  return (
    <svg
      width="32"
      height="32"
      viewBox="0 0 100 100"
      role="img"
      aria-label="Memorius"
      className="shrink-0"
    >
      <rect
        x="3"
        y="3"
        width="94"
        height="94"
        rx="22"
        className="fill-card stroke-(--glass-edge)"
      />
      <g className="stroke-t1" strokeWidth="3.6" strokeLinecap="round">
        {TICKS.map((angle) => (
          <path
            key={angle}
            d={angle % 90 === 0 ? "M50 16V23" : "M50 16V20"}
            transform={`rotate(${angle} 50 50)`}
          />
        ))}
      </g>
      <path
        d="M50 31A19 19 0 0 1 67.9 56.5"
        fill="none"
        className="stroke-attention"
        strokeWidth="6"
        strokeLinecap="round"
      />
      <circle cx="50" cy="50" r="5" className="fill-t1" />
    </svg>
  )
}
