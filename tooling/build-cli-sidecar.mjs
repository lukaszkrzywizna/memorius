// Builds the `memorius` CLI for a Rust target triple (default: this
// machine) and puts it where Tauri bundles it next to the app executable
// (`bundle.externalBin` in tauri.bundle.conf.json). A universal macOS build
// needs both architectures and their lipo'd union.
import { execFileSync } from "node:child_process"
import { copyFileSync, mkdirSync } from "node:fs"
import { join } from "node:path"
import { fileURLToPath } from "node:url"

// fileURLToPath, not URL.pathname: on Windows the latter is "/D:/...".
const root = fileURLToPath(new URL("..", import.meta.url))
const out = join(root, "apps/desktop/src-tauri/binaries")
const run = (command, args) =>
  execFileSync(command, args, { cwd: root, stdio: "inherit" })
const host = execFileSync("rustc", ["-vV"], { encoding: "utf8" })
  .split("\n")
  .find((line) => line.startsWith("host: "))
  .slice("host: ".length)
const target = process.argv[2] ?? host

const build = (triple) => {
  run("cargo", [
    "build",
    "--release",
    "--locked",
    "-p",
    "memorius-cli",
    "--target",
    triple,
  ])
  const exe = triple.includes("windows") ? ".exe" : ""
  const built = join(root, "target", triple, "release", `memorius${exe}`)
  copyFileSync(built, join(out, `memorius-${triple}${exe}`))
  return built
}

mkdirSync(out, { recursive: true })
if (target === "universal-apple-darwin") {
  const arm = build("aarch64-apple-darwin")
  const intel = build("x86_64-apple-darwin")
  run("lipo", [
    "-create",
    "-output",
    join(out, `memorius-${target}`),
    arm,
    intel,
  ])
} else {
  build(target)
}
