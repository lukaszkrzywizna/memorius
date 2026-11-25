import { describe, expect, test } from "vitest"
import { LANES, areaOf, selectLanes } from "./preflight.mjs"

describe("preflight lane selection", () => {
  test("classifies Rust, frontend, and other files", () => {
    expect(areaOf("apps/desktop/src-tauri/src/lib.rs")).toBe("rust")
    expect(areaOf("apps/desktop/src-tauri/tauri.conf.json")).toBe("rust")
    expect(areaOf("Cargo.lock")).toBe("rust")
    expect(areaOf("apps/desktop/src/main.tsx")).toBe("frontend")
    expect(areaOf("tooling/harness/preflight.mjs")).toBe("frontend")
    expect(areaOf("docs/product-specs/memorius/spec.md")).toBe("other")
  })

  test("full mode runs every lane", () => {
    expect(selectLanes(false, [])).toEqual(LANES)
  })

  test("fast mode runs only the fast lanes for staged areas", () => {
    const names = (files) => selectLanes(true, files).map((lane) => lane.name)

    expect(names(["README.md"])).toEqual(["format"])
    expect(names(["crates/x/src/lib.rs"])).toEqual(["format", "clippy"])
    expect(names(["apps/desktop/src/main.tsx"])).toEqual([
      "format",
      "oxlint",
      "eslint",
      "typecheck",
      "test",
    ])
  })
})
