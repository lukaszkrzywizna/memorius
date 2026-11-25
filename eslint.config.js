// The rules oxlint (.oxlintrc.json) does not cover: React hooks, TanStack
// Query, accessibility, tests and stories. Layer boundaries are checked by
// dependency-cruiser (apps/desktop/.dependency-cruiser.cjs).
import js from "@eslint/js"
import tanstackQuery from "@tanstack/eslint-plugin-query"
import vitest from "@vitest/eslint-plugin"
import jsxA11y from "eslint-plugin-jsx-a11y"
import oxlint from "eslint-plugin-oxlint"
import reactHooks from "eslint-plugin-react-hooks"
import storybook from "eslint-plugin-storybook"
import testingLibrary from "eslint-plugin-testing-library"
import { defineConfig } from "eslint/config"
import globals from "globals"
import tseslint from "typescript-eslint"

export default defineConfig([
  {
    ignores: [
      "**/dist",
      "**/storybook-static",
      "**/coverage",
      "**/target",
      "apps/desktop/src-tauri",
      "apps/desktop/src/shared/api/generated",
      "**/*.gen.*",
    ],
  },
  js.configs.recommended,
  tseslint.configs.recommended,
  {
    files: ["apps/desktop/src/**/*.{ts,tsx}"],
    languageOptions: { globals: globals.browser },
    extends: [
      reactHooks.configs.flat.recommended,
      jsxA11y.flatConfigs.recommended,
      tanstackQuery.configs["flat/recommended"],
    ],
  },
  {
    files: ["**/*.test.{ts,tsx}"],
    extends: [vitest.configs.recommended],
  },
  {
    files: ["apps/desktop/src/**/*.test.tsx"],
    extends: [testingLibrary.configs["flat/react"]],
  },
  storybook.configs["flat/recommended"],
  {
    files: ["**/*.{js,mjs,cjs}"],
    languageOptions: { globals: globals.node },
  },
  // oxlint already reports these; eslint stays quiet on them.
  oxlint.buildFromOxlintConfigFile("./.oxlintrc.json"),
])
