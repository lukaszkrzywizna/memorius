import type { ReactNode } from "react"

type SettingsSectionProps = {
  id: string
  title: string
  description: string
  children: ReactNode
}

/** A settings group: explanation on the left, its list on the right. */
export function SettingsSection({
  id,
  title,
  description,
  children,
}: SettingsSectionProps) {
  return (
    <section
      aria-labelledby={id}
      className="grid grid-cols-[280px_minmax(0,1fr)] gap-7 border-t border-border py-4.5 max-[960px]:grid-cols-1 max-[960px]:gap-3"
    >
      <div className="flex flex-col gap-1.5">
        <h2 id={id} className="m-0 text-[15px] font-normal text-t1">
          {title}
        </h2>
        <p className="m-0 text-[13px] leading-normal text-t4">{description}</p>
      </div>
      {children}
    </section>
  )
}
