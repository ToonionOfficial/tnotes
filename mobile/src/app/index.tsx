import * as Haptics from "expo-haptics"
import { Stack, useRouter } from "expo-router"
import { Plus, Trash2 } from "lucide-react-native"
import { useCallback, useEffect, useRef, useState } from "react"
import { Alert, Platform, Pressable, ScrollView, Text, View } from "react-native"
import { BottomBar } from "@/components/BottomBar"
import { DraggableFolderSection } from "@/components/DraggableFolderSection"
import { FolderIcon } from "@/components/FolderIcon"
import { NewFolderSheet, type NewFolderSheetRef } from "@/components/NewFolderForm"
import { VirtualFolderCard } from "@/components/VirtualFolderCard"
import type { Folder } from "@/db/schema"
import {
  useBatchDeleteFolders,
  useDeleteFolder,
  useFolders,
  useReorderFolders,
} from "@/hooks/useFolders"
import { useFolderNoteCounts } from "@/hooks/useNotes"

export default function FoldersScreen() {
  const router = useRouter()
  const { data: foldersList = [] } = useFolders()
  const deleteFolder = useDeleteFolder()
  const batchDeleteFolders = useBatchDeleteFolders()
  const reorderFolders = useReorderFolders()
  const { data: counts = { total: 0, byFolder: {}, trash: 0 } } = useFolderNoteCounts()

  const [folders, setFolders] = useState<Folder[]>(foldersList)
  useEffect(() => {
    setFolders(foldersList)
  }, [foldersList])

  const sheetRef = useRef<NewFolderSheetRef>(null)
  const [searchValue, setSearchValue] = useState("")
  const [isEditing, setIsEditing] = useState(false)
  const [selectedFolderIds, setSelectedFolderIds] = useState<Set<string>>(new Set())

  const toggleSelectFolder = useCallback((folderId: string) => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    setSelectedFolderIds((prev) => {
      const next = new Set(prev)
      if (next.has(folderId)) {
        next.delete(folderId)
      } else {
        next.add(folderId)
      }
      return next
    })
  }, [])

  const toggleEditMode = useCallback(() => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
    setIsEditing((prev) => {
      if (prev) {
        setSelectedFolderIds(new Set())
      }
      return !prev
    })
  }, [])

  const handleBatchDeleteFolders = useCallback(() => {
    const ids = Array.from(selectedFolderIds)
    if (ids.length === 0) return

    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
    const count = ids.length
    Alert.alert(
      `Delete ${count} ${count === 1 ? "Folder" : "Folders"}?`,
      `Deleting ${count === 1 ? "this folder" : "these folders"} will also move all contained notes to the Trash.`,
      [
        { text: "Cancel", style: "cancel" },
        {
          text: `Delete ${count === 1 ? "Folder" : "Folders"}`,
          style: "destructive",
          onPress: () => {
            batchDeleteFolders.mutate(ids)
            setSelectedFolderIds(new Set())
          },
        },
      ],
    )
  }, [batchDeleteFolders, selectedFolderIds])

  const handleReorder = useCallback(
    (fromIndex: number, toIndex: number) => {
      setFolders((prev) => {
        const next = [...prev]
        const [moved] = next.splice(fromIndex, 1)
        next.splice(toIndex, 0, moved)
        reorderFolders.mutate(next.map((f) => f.id))
        return next
      })
    },
    [reorderFolders],
  )

  const getFolderCount = useCallback(
    (folderId: string | null) => {
      if (folderId === null) return counts.total
      return counts.byFolder[folderId] ?? 0
    },
    [counts],
  )

  const handleDeleteFolder = useCallback(
    (folder: Folder) => {
      const noteCount = counts.byFolder[folder.id] ?? 0
      if (noteCount === 0) {
        deleteFolder.mutate(folder.id)
        return
      }

      Alert.alert(
        `Delete "${folder.name}"?`,
        `Deleting this folder will also move its ${noteCount} ${
          noteCount === 1 ? "note" : "notes"
        } to the Trash.`,
        [
          { text: "Cancel", style: "cancel" },
          {
            text: "Delete Folder",
            style: "destructive",
            onPress: () => deleteFolder.mutate(folder.id),
          },
        ],
      )
    },
    [counts.byFolder, deleteFolder],
  )

  return (
    <View className="flex-1 bg-background">
      <Stack.Screen
        options={{
          title: "Folders",
          headerLargeTitle: true,
          headerShown: true,
          unstable_headerRightItems: () =>
            isEditing
              ? [
                  {
                    type: "button" as const,
                    label:
                      selectedFolderIds.size > 0 ? `Delete (${selectedFolderIds.size})` : "Delete",
                    tintColor: selectedFolderIds.size > 0 ? "#FF3B30" : "#8E8C99",
                    sharesBackground: true,
                    onPress: handleBatchDeleteFolders,
                  },
                  {
                    type: "button" as const,
                    label: "Done",
                    tintColor: "#ffffff",
                    sharesBackground: true,
                    onPress: toggleEditMode,
                  },
                ]
              : [
                  ...(foldersList.length > 0
                    ? [
                        {
                          type: "button" as const,
                          label: "Edit",
                          tintColor: "#ffffff",
                          sharesBackground: true,
                          onPress: toggleEditMode,
                        },
                      ]
                    : []),
                  {
                    type: "button" as const,
                    label: "New Folder",
                    icon: {
                      name: "plus",
                      type: "sfSymbol" as const,
                    },
                    tintColor: "#ffffff",
                    sharesBackground: true,
                    onPress: () => {
                      void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
                      sheetRef.current?.open()
                    },
                  },
                ],
          headerRight:
            Platform.OS !== "ios"
              ? () =>
                  isEditing ? (
                    <View className="flex-row items-center">
                      <Pressable
                        onPress={handleBatchDeleteFolders}
                        disabled={selectedFolderIds.size === 0}
                        hitSlop={8}
                        className="px-2 py-1 active:opacity-60"
                      >
                        <Text
                          className={`text-[17px] font-medium ${
                            selectedFolderIds.size > 0 ? "text-[#FF3B30]" : "text-white/40"
                          }`}
                        >
                          {selectedFolderIds.size > 0
                            ? `Delete (${selectedFolderIds.size})`
                            : "Delete"}
                        </Text>
                      </Pressable>
                      <View className="mx-1 h-3.5 w-px bg-white/20" />
                      <Pressable
                        onPress={toggleEditMode}
                        hitSlop={8}
                        className="px-2 py-1 active:opacity-60"
                      >
                        <Text className="text-[17px] font-semibold text-white">Done</Text>
                      </Pressable>
                    </View>
                  ) : (
                    <View className="flex-row items-center">
                      {foldersList.length > 0 && (
                        <>
                          <Pressable
                            onPress={toggleEditMode}
                            hitSlop={8}
                            className="px-2 py-1 active:opacity-60"
                          >
                            <Text className="text-[17px] font-medium text-white">Edit</Text>
                          </Pressable>
                          <View className="mx-1 h-3.5 w-px bg-white/20" />
                        </>
                      )}
                      <Pressable
                        onPress={() => {
                          void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
                          sheetRef.current?.open()
                        }}
                        hitSlop={8}
                        className="px-2 py-1 active:opacity-60"
                      >
                        <Plus size={22} color="#ffffff" />
                      </Pressable>
                    </View>
                  )
              : undefined,
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
        <VirtualFolderCard
          title="Notes"
          icon={<FolderIcon name="folder" size={20} color="#CABEFF" fill="#CABEFF" />}
          count={getFolderCount(null)}
          isEditing={isEditing}
          onPress={() => router.push("/folders/all" as const)}
        />

        <DraggableFolderSection
          folders={folders}
          isEditing={isEditing}
          selectedFolderIds={selectedFolderIds}
          getFolderCount={getFolderCount}
          onToggleSelect={toggleSelectFolder}
          onPressFolder={(folderId) => router.push(`/folders/${folderId}` as const)}
          onDeleteFolder={handleDeleteFolder}
          onReorder={handleReorder}
        />

        <VirtualFolderCard
          title="Trash"
          icon={<Trash2 size={18} color="#FF6B6B" />}
          count={counts.trash}
          isEditing={isEditing}
          className="mt-5"
          textColor="text-[#FF6B6B]"
          onPress={() => router.push("/folders/trash" as const)}
        />
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
