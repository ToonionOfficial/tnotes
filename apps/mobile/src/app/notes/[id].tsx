import { useLocalSearchParams, useRouter } from "expo-router"
import { useMemo, useRef } from "react"
import { ActivityIndicator, View } from "react-native"
import { NoteEditor } from "@/components/editor/NoteEditor"
import {
  documentToJson,
  documentToMarkdown,
  extractText,
  firstContentText,
  markdownToDocument,
  parseDocument,
} from "@/document"
import { useFolder } from "@/hooks/useFolders"
import { useCreateNote, useNote, useUpdateNote } from "@/hooks/useNotes"
import { extractTitle, stripHtml } from "@/utils/text"

function bodyToMarkdown(body: string): string {
  if (!body.trim()) return ""
  const doc = parseDocument(body)
  if (doc) return documentToMarkdown(doc)
  return stripHtml(body)
}

export default function NoteScreen() {
  const router = useRouter()
  const { id, folderId } = useLocalSearchParams<{
    id: string
    folderId?: string
  }>()
  const isNew = id === "new"

  const { data: note, isLoading } = useNote(id)
  const activeFolderId = note?.folderId ?? folderId ?? null
  const { data: currentFolder } = useFolder(activeFolderId)
  const createNoteMutation = useCreateNote()
  const updateNoteMutation = useUpdateNote()

  const currentIdRef = useRef<string>(id)
  const initialContent = useMemo(() => bodyToMarkdown(note?.body ?? ""), [note?.body])

  const handleSave = async (markdown: string) => {
    const isBlank = !markdown.trim()
    const doc = markdownToDocument(markdown)
    const body = documentToJson(doc)
    const searchableText = extractText(doc)
    const title = extractTitle(firstContentText(doc))

    if (currentIdRef.current === "new") {
      if (isBlank) return
      const created = await createNoteMutation.mutateAsync({
        title,
        body,
        searchableText,
        folderId: folderId ?? null,
      })
      currentIdRef.current = created.id
      router.setParams({ id: created.id })
    } else {
      await updateNoteMutation.mutateAsync({
        id: currentIdRef.current,
        input: {
          title,
          body,
          searchableText,
        },
      })
    }
  }

  if (!isNew && isLoading) {
    return (
      <View className="flex-1 items-center justify-center bg-background">
        <ActivityIndicator size="large" color="#CABEFF" />
      </View>
    )
  }

  return (
    <NoteEditor
      initialContent={initialContent}
      autofocus={isNew}
      headerTitle={currentFolder?.name ?? "All Notes"}
      onSave={handleSave}
      onBack={() => router.back()}
      onDone={() => router.back()}
    />
  )
}
