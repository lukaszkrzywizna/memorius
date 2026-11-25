#!/usr/bin/env node
// Runs the repository quality gates. Lefthook and CI call this script; keep
// gate decisions here so they can be run and tested outside Git hooks.
//
//   node tooling/harness/preflight.mjs          all lanes (pre-push, CI)
//   node tooling/harness/preflight.mjs --fast   lanes for staged files (pre-commit)
//   ... --skip a,b                               leave out named lanes (CI per OS)
//
// Each lane writes its full output to .preflight/<lane>.log; failures print
// the log tail and path so a hidden hook run still shows the cause.

import { spawnSync } from "node:child_process"
import { mkdirSync, writeFileSync } from "node:fs"
import { join } from "node:path"
import { pathToFileURL } from "node:url"

const LOG_DIR = ".preflight"

/** @type {{ name: string, area: "any" | "frontend" | "rust", fast: boolean, cmd: string }[]} */
export const LANES = [
  { name: "format", area: "any", fast: true, cmd: "pnpm -s format:check" },
  { name: "oxlint", area: "frontend", fast: true, cmd: "pnpm -s lint:oxlint" },
  { name: "eslint", area: "frontend", fast: true, cmd: "pnpm -s lint:eslint" },
  { name: "typecheck", area: "frontend", fast: true, cmd: "pnpm -s typecheck" },
  {
    name: "test",
    area: "frontend",
    fast: true,
    cmd: "pnpm -s -C apps/desktop test",
  },
  {
    name: "harness-test",
    area: "frontend",
    fast: false,
    cmd: "pnpm -s test:harness",
  },
  {
    name: "architecture",
    area: "frontend",
    fast: false,
    cmd: "pnpm -s check:frontend:architecture",
  },
  { name: "knip", area: "frontend", fast: false, cmd: "pnpm -s knip" },
  {
    name: "web-build",
    area: "frontend",
    fast: false,
    cmd: "pnpm -s -C apps/desktop build",
  },
  {
    name: "storybook-build",
    area: "frontend",
    fast: false,
    cmd: "pnpm -s -C apps/desktop build:storybook --quiet",
  },
  { name: "clippy", area: "rust", fast: true, cmd: "pnpm -s lint:rust" },
  { name: "rust-test", area: "rust", fast: false, cmd: "pnpm -s test:rust" },
  { name: "cargo-deny", area: "rust", fast: false, cmd: "pnpm -s deny" },
]

/** @param {string} file */
export function areaOf(file) {
  if (
    /\.rs$|(^|\/)Cargo\.(toml|lock)$|^deny\.toml$|^rust-toolchain\.toml$|src-tauri\//.test(
      file,
    )
  ) {
    return "rust"
  }
  if (/\.(ts|tsx|js|mjs|cjs|css|html|json|jsonc|ya?ml)$/.test(file)) {
    return "frontend"
  }
  return "other"
}

/** @param {boolean} fast @param {string[]} stagedFiles */
export function selectLanes(fast, stagedFiles) {
  if (!fast) return LANES
  const areas = new Set(stagedFiles.map(areaOf))
  return LANES.filter(
    (lane) => lane.fast && (lane.area === "any" || areas.has(lane.area)),
  )
}

function stagedFiles() {
  const result = spawnSync(
    "git",
    ["diff", "--cached", "--name-only", "--diff-filter=ACMR"],
    {
      encoding: "utf8",
    },
  )
  return result.stdout.split("\n").filter(Boolean)
}

function run() {
  const fast = process.argv.includes("--fast")
  const skipIndex = process.argv.indexOf("--skip")
  const skip = new Set(
    skipIndex === -1 ? [] : (process.argv[skipIndex + 1] ?? "").split(","),
  )
  const lanes = selectLanes(fast, fast ? stagedFiles() : []).filter(
    (lane) => !skip.has(lane.name),
  )
  mkdirSync(LOG_DIR, { recursive: true })
  const failed = []
  const started = Date.now()

  for (const lane of lanes) {
    const laneStart = Date.now()
    const result = spawnSync(lane.cmd, {
      shell: true,
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
    })
    const output = `${result.stdout ?? ""}${result.stderr ?? ""}`
    const logPath = join(LOG_DIR, `${lane.name}.log`)
    writeFileSync(logPath, output)
    const seconds = ((Date.now() - laneStart) / 1000).toFixed(1)
    if (result.status === 0) {
      console.log(`PASS ${lane.name} (${seconds}s)`)
    } else {
      failed.push(lane.name)
      console.log(`FAIL ${lane.name} (${seconds}s) — full log: ${logPath}`)
      console.log(output.split("\n").slice(-40).join("\n"))
    }
  }

  const total = ((Date.now() - started) / 1000).toFixed(1)
  console.log(
    failed.length === 0
      ? `Preflight passed: ${lanes.length} lane(s) in ${total}s`
      : `Preflight failed: ${failed.join(", ")} (${total}s)`,
  )
  return failed.length === 0 ? 0 : 1
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  process.exitCode = run()
}
