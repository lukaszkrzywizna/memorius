#!/usr/bin/env node
// Fails when `cargo test` changed the ts-rs bindings: the Rust API types and
// the committed TypeScript types must match.

import { spawnSync } from "node:child_process"

const dir = "apps/desktop/src/shared/api/generated"
// Compare the working tree with the index, so staged bindings count as
// committed during the pre-commit and pre-push hooks.
const git = (args) => spawnSync("git", args, { encoding: "utf8" })
const modified = git(["diff", "--name-only", "--", dir])
const untracked = git(["ls-files", "--others", "--exclude-standard", "--", dir])
const changed = `${modified.stdout}${untracked.stdout}`.trim()

if (modified.status !== 0 || untracked.status !== 0 || changed) {
  console.error(
    `TypeScript bindings in ${dir} differ from the Rust types. Run \`cargo test --workspace\` and commit the result.\n${changed}`,
  )
  process.exitCode = 1
}
