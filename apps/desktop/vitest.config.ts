import { configDefaults, defineConfig, mergeConfig } from "vitest/config"
import viteConfig from "./vite.config"

export default mergeConfig(
  viteConfig,
  defineConfig({
    test: {
      environment: "jsdom",
      setupFiles: ["./src/app/testing/vitest.setup.ts"],
      exclude: [...configDefaults.exclude],
      restoreMocks: true,
      coverage: {
        provider: "v8",
        reporter: ["text-summary", "lcov"],
      },
    },
  }),
)
