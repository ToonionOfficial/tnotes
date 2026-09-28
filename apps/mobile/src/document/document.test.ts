import { describe, expect, it } from "vitest"
import { documentToJson, emptyDocument, paragraphDocument, parseDocument } from "@/document/codec"
import { extractText } from "@/document/extract"

describe("parseDocument", () => {
  it("returns an empty document for blank input", () => {
    expect(parseDocument("")).toEqual({ version: 1, blocks: [] })
    expect(parseDocument("   ")).toEqual({ version: 1, blocks: [] })
  })

  it("defaults a missing version to 1", () => {
    const doc = parseDocument({ blocks: [] })
    expect(doc).toEqual({ version: 1, blocks: [] })
  })

  it("backfills missing block ids", () => {
    const doc = parseDocument({
      blocks: [{ type: "Paragraph", data: { spans: [] } }],
    })
    expect(doc?.blocks).toHaveLength(1)
    expect(doc?.blocks[0]?.id).toMatch(/^[0-9A-HJKMNP-TV-Z]{26}$/)
  })

  it("keeps stable block ids", () => {
    const raw = {
      version: 1,
      blocks: [{ id: "01J0000000000000000000000", type: "Divider" }],
    }
    expect(parseDocument(raw)).toEqual(raw)
  })

  it("rejects unknown block types and non-objects", () => {
    expect(parseDocument({ blocks: [{ type: "Nope", data: {} }] })).toBeUndefined()
    expect(parseDocument("{invalid")).toBeUndefined()
    expect(parseDocument(42)).toBeUndefined()
    expect(parseDocument({})).toBeUndefined()
  })

  it("roundtrips through json", () => {
    const doc = paragraphDocument("hello")
    expect(parseDocument(documentToJson(doc))).toEqual(doc)
  })
})

describe("extractText", () => {
  it("returns empty string for an empty document", () => {
    expect(extractText(emptyDocument())).toBe("")
  })

  it("matches the canonical extraction order and skips binary payloads", () => {
    const doc = parseDocument({
      version: 1,
      blocks: [
        {
          id: "01J0000000000000000000001",
          type: "Heading",
          data: { level: 1, content: { spans: [{ text: "Meeting Notes" }] } },
        },
        {
          id: "01J0000000000000000000002",
          type: "Paragraph",
          data: { spans: [{ text: "Discussed roadmap." }] },
        },
        {
          id: "01J0000000000000000000003",
          type: "Table",
          data: {
            headers: ["Feature", "Status"],
            rows: [["FTS", "Done"]],
          },
        },
        {
          id: "01J0000000000000000000004",
          type: "TaskList",
          data: [
            { checked: true, content: { spans: [{ text: "Write tests" }] } },
            { checked: false, content: { spans: [{ text: "Ship release" }] } },
          ],
        },
        {
          id: "01J0000000000000000000005",
          type: "Image",
          data: { asset_id: "img1", alt: "Architecture diagram" },
        },
        { id: "01J0000000000000000000006", type: "Divider" },
        {
          id: "01J0000000000000000000007",
          type: "Drawing",
          data: { strokes: [] },
        },
        {
          id: "01J0000000000000000000008",
          type: "Audio",
          data: { asset_id: "a1", duration_ms: 1000 },
        },
      ],
    })
    expect(doc).toBeDefined()
    if (!doc) return
    expect(extractText(doc)).toBe(
      "Meeting Notes Discussed roadmap. Feature Status FTS Done Write tests Ship release Architecture diagram",
    )
  })

  it("recurses into nested lists", () => {
    const doc = parseDocument({
      blocks: [
        {
          id: "01J0000000000000000000009",
          type: "BulletList",
          data: [
            {
              content: { spans: [{ text: "outer" }] },
              sub_list: {
                Bullet: [{ content: { spans: [{ text: "inner" }] } }],
              },
            },
          ],
        },
      ],
    })
    expect(doc).toBeDefined()
    if (!doc) return
    expect(extractText(doc)).toBe("outer inner")
  })
})
