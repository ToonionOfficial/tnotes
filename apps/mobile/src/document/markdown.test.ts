import { describe, expect, it } from "vitest"
import { parseDocument } from "@/document/codec"
import { extractText } from "@/document/extract"
import { documentToMarkdown, markdownToDocument } from "@/document/markdown"

describe("documentToMarkdown", () => {
  it("serializes headings, paragraphs and marks", () => {
    const doc = parseDocument({
      version: 1,
      blocks: [
        {
          id: "01J0000000000000000000001",
          type: "Heading",
          data: { level: 1, content: { spans: [{ text: "Shopping" }] } },
        },
        {
          id: "01J0000000000000000000002",
          type: "Paragraph",
          data: {
            spans: [
              { text: "Buy " },
              { text: "milk", marks: { bold: true } },
              { text: " and eggs" },
            ],
          },
        },
      ],
    })
    expect(doc).toBeDefined()
    if (!doc) return
    expect(documentToMarkdown(doc)).toBe("# Shopping\n\nBuy **milk** and eggs")
  })

  it("serializes task lists, code blocks and dividers", () => {
    const doc = parseDocument({
      version: 1,
      blocks: [
        {
          id: "01J0000000000000000000001",
          type: "TaskList",
          data: [
            { checked: true, content: { spans: [{ text: "Write tests" }] } },
            { checked: false, content: { spans: [{ text: "Ship" }] } },
          ],
        },
        {
          id: "01J0000000000000000000002",
          type: "CodeBlock",
          data: { language: "ts", code: "const a = 1" },
        },
        { id: "01J0000000000000000000003", type: "Divider" },
      ],
    })
    expect(doc).toBeDefined()
    if (!doc) return
    expect(documentToMarkdown(doc)).toBe(
      "- [x] Write tests\n- [ ] Ship\n\n```ts\nconst a = 1\n```\n\n---",
    )
  })

  it("serializes tables and images", () => {
    const doc = parseDocument({
      version: 1,
      blocks: [
        {
          id: "01J0000000000000000000001",
          type: "Table",
          data: { headers: ["A", "B"], rows: [["1", "2"]] },
        },
        {
          id: "01J0000000000000000000002",
          type: "Image",
          data: { asset_id: "img1", alt: "Diagram" },
        },
      ],
    })
    expect(doc).toBeDefined()
    if (!doc) return
    expect(documentToMarkdown(doc)).toBe(
      "| A | B |\n| --- | --- |\n| 1 | 2 |\n\n![Diagram](asset://img1)",
    )
  })
})

describe("markdownToDocument", () => {
  it("parses headings, paragraphs and inline marks", () => {
    const doc = markdownToDocument("# Shopping\n\nBuy **milk** and eggs")
    expect(doc.blocks).toHaveLength(2)
    expect(doc.blocks[0]).toMatchObject({
      type: "Heading",
      data: { level: 1, content: { spans: [{ text: "Shopping" }] } },
    })
    expect(doc.blocks[1]?.type).toBe("Paragraph")
    if (doc.blocks[1]?.type !== "Paragraph") return
    expect(doc.blocks[1].data.spans).toEqual([
      expect.objectContaining({ text: "Buy " }),
      expect.objectContaining({ text: "milk", marks: expect.objectContaining({ bold: true }) }),
      expect.objectContaining({ text: " and eggs" }),
    ])
  })

  it("parses task lists and promotes mixed lists", () => {
    const doc = markdownToDocument("- [x] Done\n- [ ] Todo")
    expect(doc.blocks).toHaveLength(1)
    expect(doc.blocks[0]).toMatchObject({
      type: "TaskList",
      data: [
        { checked: true, content: { spans: [{ text: "Done" }] } },
        { checked: false, content: { spans: [{ text: "Todo" }] } },
      ],
    })
  })

  it("parses quotes, code blocks and dividers", () => {
    const doc = markdownToDocument("> quoted\n\n```ts\ncode\n```\n\n---")
    expect(doc.blocks.map((block) => block.type)).toEqual(["Quote", "CodeBlock", "Divider"])
    expect(doc.blocks[1]).toMatchObject({
      data: { language: "ts", code: "code" },
    })
  })

  it("keeps drawing and audio directives as blocks", () => {
    const doc = markdownToDocument(
      '<!-- tnotes:drawing asset_id="d1" -->\n<!-- tnotes:audio asset_id="a1" duration_ms="500" -->',
    )
    expect(doc.blocks).toMatchObject([
      { type: "Drawing", data: { asset_id: "d1" } },
      { type: "Audio", data: { asset_id: "a1", duration_ms: 500 } },
    ])
  })

  it("roundtrips visible text through markdown", () => {
    const original = markdownToDocument("hello **world**")
    const markdown = documentToMarkdown(original)
    const back = markdownToDocument(markdown)
    expect(extractText(back)).toBe(extractText(original))
    expect(documentToMarkdown(back)).toBe(markdown)
  })
})
