import { useState } from "react"
import type { AppRef } from "@/shared/api/generated/AppRef"
import { appIconUrl } from "@/entities/activity/lib/app-icon-url"

type AppIconProps = { app: AppRef; size?: number }

/** The app's own icon; its first letter on a neutral tile when the OS has none. */
export function AppIcon({ app, size = 16 }: AppIconProps) {
  return <AppIconImage key={app.id} app={app} size={size} />
}

function AppIconImage({ app, size }: Required<AppIconProps>) {
  const [missing, setMissing] = useState(false)
  if (!missing) {
    return (
      <img
        src={appIconUrl(app.id)}
        alt=""
        width={size}
        height={size}
        // Flex rows would stretch an image to their height otherwise.
        style={{ width: size, height: size }}
        className="shrink-0"
        onError={() => setMissing(true)}
      />
    )
  }
  return (
    <span
      aria-hidden="true"
      className="inline-flex shrink-0 items-center justify-center rounded-[22%] bg-accent text-t3"
      style={{
        width: size,
        height: size,
        fontSize: Math.round(size * 0.6),
      }}
    >
      {app.name.charAt(0).toUpperCase()}
    </span>
  )
}
