import { useLocalSearchParams } from "expo-router"
import { NoteEditor } from "@/components/editor/NoteEditor"

export default function NoteScreen() {
  const { id } = useLocalSearchParams<{ id: string }>()
  const isNew = id === "new"

  return (
    <NoteEditor
      placeholder={isNew ? "Title..." : "Start writing..."}
      autofocus={isNew}
      headerTitle="Notes"
    />
  )
}
