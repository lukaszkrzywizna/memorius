import { afterEach, describe, expect, it } from "vitest"
import { readStoredZoom, storeZoom } from "@/pages/day/model/zoom-storage"

describe("zoom storage", () => {
  afterEach(() => localStorage.clear())

  it("reads back the zoom it stored", () => {
    storeZoom(60)

    expect(readStoredZoom()).toBe(60)
  })

  it("discards values the zoom does not offer", () => {
    localStorage.setItem("memorius.day.zoom", "7")

    expect(readStoredZoom()).toBeUndefined()
  })
})
