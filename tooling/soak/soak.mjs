#!/usr/bin/env node
// Memory soak: starts Memorius hidden (as the login item does), samples its
// private memory every interval, and fails on growth or on the budget.
//
//   node tooling/soak/soak.mjs --binary target/release/memorius-desktop --minutes 480
//
// Private memory = macOS phys_footprint (Activity Monitor "Memory") or the
// Windows private working set. RSS is not used: it counts shared system
// frameworks (AppKit, WebKit) that every app maps.

import { execFileSync, spawn } from "node:child_process"
import { parseArgs } from "node:util"

const { values } = parseArgs({
  options: {
    binary: { type: "string" },
    minutes: { type: "string", default: "10" },
    interval: { type: "string", default: "60" },
    "budget-mb": { type: "string", default: "60" },
    "max-growth-mb": { type: "string", default: "8" },
  },
})
if (!values.binary) {
  console.error(
    "Usage: soak.mjs --binary <path> [--minutes 10] [--interval 60]",
  )
  process.exit(2)
}

/** @param {number} pid */
function privateMb(pid) {
  if (process.platform === "darwin") {
    const out = execFileSync("footprint", [String(pid)], { encoding: "utf8" })
    const match = /Footprint: ([\d.]+) (KB|MB|GB)/.exec(out)
    if (!match) throw new Error(`Cannot read footprint:\n${out}`)
    const scale = { KB: 1 / 1024, MB: 1, GB: 1024 }[match[2] ?? "MB"] ?? 1
    return Number(match[1]) * scale
  }
  const out = execFileSync(
    "powershell",
    ["-NoProfile", "-Command", `(Get-Process -Id ${pid}).PrivateMemorySize64`],
    { encoding: "utf8" },
  )
  return Number(out.trim()) / 1024 / 1024
}

const app = spawn(values.binary, ["--hidden"], { stdio: "ignore" })
const intervalMs = Number(values.interval) * 1000
const samples = Math.max(
  2,
  Math.floor((Number(values.minutes) * 60_000) / intervalMs),
)
const readings = []
console.log("minute,private_mb")

for (let index = 0; index <= samples; index += 1) {
  await new Promise((resolve) =>
    setTimeout(resolve, index === 0 ? 10_000 : intervalMs),
  )
  const mb = privateMb(app.pid ?? 0)
  readings.push(mb)
  console.log(`${((index * intervalMs) / 60_000).toFixed(1)},${mb.toFixed(1)}`)
}
app.kill()

const first = readings[0] ?? 0
const peak = Math.max(...readings)
const growth = (readings.at(-1) ?? 0) - first
const failures = [
  peak > Number(values["budget-mb"]) &&
    `peak ${peak.toFixed(1)} MB is over the ${values["budget-mb"]} MB budget`,
  growth > Number(values["max-growth-mb"]) &&
    `memory grew ${growth.toFixed(1)} MB (limit ${values["max-growth-mb"]} MB)`,
].filter(Boolean)
console.log(
  failures.length === 0
    ? `Soak passed: start ${first.toFixed(1)} MB, peak ${peak.toFixed(1)} MB, growth ${growth.toFixed(1)} MB`
    : `Soak failed: ${failures.join("; ")}`,
)
process.exitCode = failures.length === 0 ? 0 : 1
