import { Stack, useRouter } from "expo-router"
import { ChevronRight, Plus, Trash2 } from "lucide-react-native"
import { useRef, useState } from "react"
import { Pressable, ScrollView, Text, View } from "react-native"
import { BottomBar } from "@/components/BottomBar"
import { DEFAULT_FOLDER_ICON, FolderIcon } from "@/components/FolderIcon"
import { NewFolderSheet, type NewFolderSheetRef } from "@/components/NewFolderForm"
import type { Folder } from "@/db/schema"
import { useFolders } from "@/hooks/useFolders"
import { useNotes } from "@/hooks/useNotes"

export default function FoldersScreen() {
  const router = useRouter()
  const { data: foldersList = [] } = useFolders()
  const { data: allNotes = [] } = useNotes()
  const { data: trashedNotes = [] } = useNotes({ trashed: true })

  const sheetRef = useRef<NewFolderSheetRef>(null)
  const [searchValue, setSearchValue] = useState("")

  const getFolderCount = (folderId: string | null) => {
    if (folderId === null) return allNotes.length
    return allNotes.filter((n) => n.folderId === folderId).length
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
              onPress={() => sheetRef.current?.open()}
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
        <View className="mb-5 overflow-hidden rounded-2xl bg-white/[0.07]">
          <Pressable
            onPress={() => router.push("/folders/all" as const)}
            className="flex-row items-center justify-between px-4 py-3.5 active:bg-white/12"
          >
            <View className="flex-row items-center gap-3">
              <View className="size-8 items-center justify-center rounded-lg">
                <FolderIcon name="folder" size={20} color="#CABEFF" fill="#CABEFF" />
              </View>
              <Text className="text-[17px] text-foreground">All Notes</Text>
            </View>
            <View className="flex-row items-center gap-1.5">
              <Text className="text-[15px] text-muted-foreground">{getFolderCount(null)}</Text>
              <ChevronRight size={16} color="#8E8C99" />
            </View>
          </Pressable>

          {foldersList.map((folder: Folder) => (
            <View key={folder.id}>
              <View className="ml-15 h-[0.5px] bg-white/8" />
              <Pressable
                onPress={() => router.push(`/folders/${folder.id}` as const)}
                className="flex-row items-center justify-between px-4 py-3.5 active:bg-white/12"
              >
                <View className="flex-row items-center gap-3">
                  <View className="size-8 items-center justify-center rounded-lg bg-white/6">
                    <FolderIcon name={folder.icon || DEFAULT_FOLDER_ICON} size={18} color="#CABEFF" />
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

        <View className="overflow-hidden rounded-2xl bg-white/[0.07]">
          <Pressable
            onPress={() => router.push("/folders/trash" as const)}
            className="flex-row items-center justify-between px-4 py-3.5 active:bg-white/12"
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
        onPressNewNote={() => router.push("/notes/new" as const)}
      />

      <NewFolderSheet
        ref={sheetRef}
        onCreated={(folderId) => router.push(`/folders/${folderId}` as const)}
      />
    </View>
  )
}
