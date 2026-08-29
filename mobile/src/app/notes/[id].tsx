import { useLocalSearchParams, useRouter } from "expo-router"
import { useRef } from "react"
import { ActivityIndicator, View } from "react-native"
import { NoteEditor } from "@/components/editor/NoteEditor"
import { useFolder } from "@/hooks/useFolders"
import { useCreateNote, useNote, useUpdateNote } from "@/hooks/useNotes"
import { extractTitle } from "@/utils/text"

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

  const handleSave = async (html: string, plainText: string) => {
    const isBlank = !html.trim() || html === "<p></p>"
    const title = extractTitle(plainText)

    if (currentIdRef.current === "new") {
      if (isBlank) return
      const created = await createNoteMutation.mutateAsync({
        title,
        body: html,
        folderId: folderId ?? null,
      })
      currentIdRef.current = created.id
      router.setParams({ id: created.id })
    } else {
      await updateNoteMutation.mutateAsync({
        id: currentIdRef.current,
        input: {
          title,
          body: html,
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
      initialContent={note?.body ?? ""}
      autofocus={isNew}
      headerTitle={currentFolder?.name ?? "All Notes"}
      onSave={handleSave}
      onBack={() => router.back()}
      onDone={() => router.back()}
    />
  )
}
