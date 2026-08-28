import { useState } from "react"
import { Keyboard, Pressable, Text, TextInput, View } from "react-native"
import { useCreateFolder } from "@/hooks/useFolders"

const EMOJI_OPTIONS = ["📁", "💼", "💡", "📝", "🚀", "🎯", "📚", "🎨", "🏠", "💰"]

interface NewFolderFormProps {
  onCreated: (folderId: string) => void
  onCancel: () => void
}

export function NewFolderForm({ onCreated, onCancel }: NewFolderFormProps) {
  const [newFolderName, setNewFolderName] = useState("")
  const [selectedEmoji, setSelectedEmoji] = useState("📁")
  const createFolderMutation = useCreateFolder()

  const resetForm = () => {
    setNewFolderName("")
    setSelectedEmoji("📁")
    Keyboard.dismiss()
    onCancel()
  }

  const handleCreateFolder = async () => {
    const trimmed = newFolderName.trim()
    if (!trimmed) return

    const created = await createFolderMutation.mutateAsync({
      name: trimmed,
      icon: selectedEmoji,
    })

    setNewFolderName("")
    setSelectedEmoji("📁")
    Keyboard.dismiss()
    onCreated(created.id)
  }

  return (
    <View className="mb-5 overflow-hidden rounded-2xl bg-white/[0.07]">
      <View className="flex-row items-center px-4 py-3">
        <Text className="mr-3 text-2xl">{selectedEmoji}</Text>
        <TextInput
          value={newFolderName}
          onChangeText={setNewFolderName}
          placeholder="New Folder"
          placeholderTextColor="#6E6B77"
          cursorColor="#CABEFF"
          autoFocus
          className="flex-1 text-[17px] font-medium text-foreground"
          onSubmitEditing={handleCreateFolder}
          returnKeyType="done"
        />
      </View>
      <View className="mx-4 h-[0.5px] bg-white/[0.08]" />
      <View className="flex-row items-center justify-around px-3 py-2.5">
        {EMOJI_OPTIONS.map((emoji) => (
          <Pressable
            key={emoji}
            onPress={() => setSelectedEmoji(emoji)}
            className={`size-9 items-center justify-center rounded-xl ${
              selectedEmoji === emoji ? "bg-white/15" : "active:bg-white/10"
            }`}
          >
            <Text className="text-[18px]">{emoji}</Text>
          </Pressable>
        ))}
      </View>
      <View className="mx-4 h-[0.5px] bg-white/[0.08]" />
      <View className="flex-row items-center justify-end gap-3 px-4 py-2.5">
        <Pressable onPress={resetForm} className="rounded-xl px-4 py-2 active:bg-white/[0.06]">
          <Text className="text-[15px] font-medium text-muted-foreground">Cancel</Text>
        </Pressable>
        <Pressable
          onPress={handleCreateFolder}
          disabled={!newFolderName.trim()}
          className={`rounded-xl bg-primary px-5 py-2 active:opacity-80 ${
            !newFolderName.trim() ? "opacity-30" : ""
          }`}
        >
          <Text className="text-[15px] font-semibold text-[#141318]">Create</Text>
        </Pressable>
      </View>
    </View>
  )
}
