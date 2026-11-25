import { createHashHistory, createRouter } from "@tanstack/react-router"
import { routeTree } from "@/routeTree.gen"

// Tauri serves one index.html; hash routes also let the popover window open
// at `index.html#/popover`.
export const router = createRouter({
  routeTree,
  history: createHashHistory(),
  defaultPreload: "intent",
})

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router
  }
}
