import { Stack, useRouter } from "expo-router"
import { ChevronRight, Folder as FolderIcon, Plus, Trash2 } from "lucide-react-native"
import { useCallback, useState } from "react"
import { Keyboard, Pressable, ScrollView, Text, TextInput, View } from "react-native"
import { BottomBar } from "@/components/BottomBar"
import type { Folder } from "@/db/schema"
import { useCreateFolder, useFolders } from "@/hooks/useFolders"
import { useNotes } from "@/hooks/useNotes"

const EMOJI_OPTIONS = ["📁", "💼", "💡", "📝", "🚀", "🎯", "📚", "🎨", "🏠", "💰"]

export default function FoldersScreen() {
  const router = useRouter()
  const { data: foldersList = [] } = useFolders()
  const { data: allNotes = [] } = useNotes()
  const { data: trashedNotes = [] } = useNotes({ trashed: true })
  const createFolderMutation = useCreateFolder()

  const [isCreating, setIsCreating] = useState(false)
  const [newFolderName, setNewFolderName] = useState("")
  const [selectedEmoji, setSelectedEmoji] = useState("📁")
  const [searchValue, setSearchValue] = useState("")

  const resetForm = useCallback(() => {
    setIsCreating(false)
    setNewFolderName("")
    setSelectedEmoji("📁")
    Keyboard.dismiss()
  }, [])

  const handleCreateFolder = async () => {
    const trimmed = newFolderName.trim()
    if (!trimmed) return

    const created = await createFolderMutation.mutateAsync({
      name: trimmed,
      icon: selectedEmoji,
    })

    resetForm()
    router.push(`/folders/${created.id}` as const)
  }

  const getFolderCount = (folderId: string | null) => {
    if (folderId === null) return allNotes.length
    return allNotes.filter((n) => n.folderId === folderId).length
  }

  const handlePressNewNote = () => {
    router.push("/notes/new" as const)
  }

  return (
    <View className="flex-1 bg-background">
      <Stack.Screen
        options={{
          title: "Folders",
          headerLargeTitle: true,
          headerShown: true,
          headerRight: () => (
            <Pressable
              onPress={() => setIsCreating((v) => !v)}
              hitSlop={8}
              className="active:opacity-60"
            >
              <Plus size={22} color="#CABEFF" />
            </Pressable>
          ),
        }}
      />

      <ScrollView
        className="flex-1"
        contentInsetAdjustmentBehavior="automatic"
        keyboardShouldPersistTaps="handled"
        contentContainerStyle={{
          paddingHorizontal: 20,
          paddingTop: 12,
          paddingBottom: 110,
        }}
      >
        {/* New Folder Form — clean inline row that blends into the list */}
        {isCreating && (
          <View className="mb-5 overflow-hidden rounded-2xl bg-white/[0.07]">
            {/* Icon + Name Input */}
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

            {/* Emoji Row */}
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

            {/* Action Buttons */}
            <View className="mx-4 h-[0.5px] bg-white/[0.08]" />
            <View className="flex-row items-center justify-end gap-3 px-4 py-2.5">
              <Pressable
                onPress={resetForm}
                className="rounded-xl px-4 py-2 active:bg-white/[0.06]"
              >
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
        )}

        {/* Main Folders Group */}
        <View className="mb-5 overflow-hidden rounded-2xl bg-white/[0.07]">
          {/* All Notes */}
          <Pressable
            onPress={() => router.push("/folders/all" as const)}
            className="flex-row items-center justify-between px-4 py-3.5 active:bg-white/[0.12]"
          >
            <View className="flex-row items-center gap-3">
              <View className="size-8 items-center justify-center rounded-lg bg-primary/15">
                <FolderIcon size={18} color="#CABEFF" />
              </View>
              <Text className="text-[17px] text-foreground">All Notes</Text>
            </View>
            <View className="flex-row items-center gap-1.5">
              <Text className="text-[15px] text-muted-foreground">{getFolderCount(null)}</Text>
              <ChevronRight size={16} color="#8E8C99" />
            </View>
          </Pressable>

          {/* Custom Folders */}
          {foldersList.map((folder: Folder) => (
            <View key={folder.id}>
              <View className="ml-[60px] h-[0.5px] bg-white/[0.08]" />
              <Pressable
                onPress={() => router.push(`/folders/${folder.id}` as const)}
                className="flex-row items-center justify-between px-4 py-3.5 active:bg-white/[0.12]"
              >
                <View className="flex-row items-center gap-3">
                  <View className="size-8 items-center justify-center rounded-lg bg-white/[0.06]">
                    <Text className="text-[18px]">{folder.icon || "📁"}</Text>
                  </View>
                  <Text className="text-[17px] text-foreground">{folder.name}</Text>
                </View>
                <View className="flex-row items-center gap-1.5">
                  <Text className="text-[15px] text-muted-foreground">
                    {getFolderCount(folder.id)}
                  </Text>
                  <ChevronRight size={16} color="#8E8C99" />
                </View>
              </Pressable>
            </View>
          ))}
        </View>

        {/* Trash */}
        <View className="overflow-hidden rounded-2xl bg-white/[0.07]">
          <Pressable
            onPress={() => router.push("/folders/trash" as const)}
            className="flex-row items-center justify-between px-4 py-3.5 active:bg-white/[0.12]"
          >
            <View className="flex-row items-center gap-3">
              <View className="size-8 items-center justify-center rounded-lg bg-[#FF6B6B]/10">
                <Trash2 size={16} color="#FF6B6B" />
              </View>
              <Text className="text-[17px] text-[#FF6B6B]">Trash</Text>
            </View>
            <View className="flex-row items-center gap-1.5">
              <Text className="text-[15px] text-muted-foreground">{trashedNotes.length}</Text>
              <ChevronRight size={16} color="#8E8C99" />
            </View>
          </Pressable>
        </View>
      </ScrollView>

      <BottomBar
        searchValue={searchValue}
        onSearchChange={setSearchValue}
        onPressNewNote={handlePressNewNote}
      />
    </View>
  )
}
