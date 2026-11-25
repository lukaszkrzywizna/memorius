import type { Title } from "@/shared/api/generated/Title"

/** Text for a window title; missing titles say why instead of showing nothing. */
export function windowTitle(title: Title) {
  switch (title.kind) {
    case "known":
      return title.text.trim() === "" ? "Untitled window" : title.text
    case "unavailable":
      switch (title.reason) {
        case "permissionDenied":
          return "Title hidden (Accessibility not allowed)"
        case "noWindow":
          return "No window"
        case "private":
          return "Private app"
      }
  }
}
