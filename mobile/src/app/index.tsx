import * as Haptics from "expo-haptics"
import { Stack, useRouter } from "expo-router"
import { ChevronRight, Circle, CircleCheck, GripVertical, Plus, Trash2 } from "lucide-react-native"
import { useCallback, useMemo, useRef, useState } from "react"
import { Alert, FlatList, Pressable, Text, View } from "react-native"
import { BottomBar } from "@/components/BottomBar"
import { DEFAULT_FOLDER_ICON, FolderIcon } from "@/components/FolderIcon"
import { NewFolderSheet, type NewFolderSheetRef } from "@/components/NewFolderForm"
import { SwipeableListItem } from "@/components/SwipeableListItem"
import type { Folder } from "@/db/schema"
import { useDeleteFolder, useFolders } from "@/hooks/useFolders"
import { useFolderNoteCounts } from "@/hooks/useNotes"

type FolderItem =
  | { type: "all"; id: "all" }
  | { type: "folder"; id: string; folder: Folder; isFirst: boolean; isLast: boolean }
  | { type: "trash"; id: "trash" }

export default function FoldersScreen() {
  const router = useRouter()
  const { data: foldersList = [] } = useFolders()
  const deleteFolder = useDeleteFolder()
  const { data: counts = { total: 0, byFolder: {}, trash: 0 } } = useFolderNoteCounts()

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

  const listData = useMemo<FolderItem[]>(() => {
    const items: FolderItem[] = [{ type: "all", id: "all" }]
    foldersList.forEach((folder, index) => {
      items.push({
        type: "folder",
        id: folder.id,
        folder,
        isFirst: index === 0,
        isLast: index === foldersList.length - 1,
      })
    })
    items.push({ type: "trash", id: "trash" })
    return items
  }, [foldersList])

  const renderItem = useCallback(
    ({ item }: { item: FolderItem }) => {
      if (item.type === "all") {
        return (
          <View
            className={`overflow-hidden rounded-3xl bg-white/7 ${isEditing ? "opacity-40" : ""}`}
          >
            <Pressable
              disabled={isEditing}
              onPress={() => router.push("/folders/all" as const)}
              className="flex-row items-center justify-between px-4 py-3.5 active:bg-white/12"
            >
              <View className="flex-row items-center gap-3">
                <View className="size-8 items-center justify-center rounded-lg">
                  <FolderIcon name="folder" size={20} color="#CABEFF" fill="#CABEFF" />
                </View>
                <Text className="text-[17px] text-foreground">Notes</Text>
              </View>
              <View className="flex-row items-center gap-1.5">
                <Text className="text-[15px] text-muted-foreground">{getFolderCount(null)}</Text>
                <ChevronRight size={16} color="#8E8C99" />
              </View>
            </Pressable>
          </View>
        )
      }

      if (item.type === "folder") {
        const isOnly = item.isFirst && item.isLast
        const isSelected = selectedFolderIds.has(item.folder.id)
        const roundingClass = isOnly
          ? "rounded-3xl"
          : item.isFirst
            ? "rounded-t-3xl"
            : item.isLast
              ? "rounded-b-3xl"
              : ""

        return (
          <View
            className={`overflow-hidden bg-white/7 ${roundingClass} ${item.isFirst ? "mt-5" : ""}`}
          >
            {!item.isFirst && <View className="ml-15 h-[0.5px] bg-white/8" />}
            <SwipeableListItem
              enabled={!isEditing}
              rounded={isOnly ? "only" : item.isFirst ? "first" : item.isLast ? "last" : "middle"}
              rightAction={{
                label: "Delete",
                color: "#D94C5C",
                icon: <Trash2 size={20} color="#FFFFFF" />,
                onPress: () => handleDeleteFolder(item.folder),
              }}
            >
              <Pressable
                onPress={() => {
                  if (isEditing) {
                    toggleSelectFolder(item.folder.id)
                  } else {
                    router.push(`/folders/${item.folder.id}` as const)
                  }
                }}
                className={`flex-row items-center justify-between px-4 py-3.5 active:bg-white/12 ${roundingClass}`}
              >
                <View className="flex-row items-center gap-3">
                  {isEditing && (
                    <View className="mr-0.5">
                      {isSelected ? (
                        <CircleCheck size={22} color="#CABEFF" fill="#CABEFF" />
                      ) : (
                        <Circle size={22} color="#8E8C99" />
                      )}
                    </View>
                  )}
                  <View className="size-8 items-center justify-center rounded-lg">
                    <FolderIcon
                      name={item.folder.icon || DEFAULT_FOLDER_ICON}
                      size={20}
                      color="#CABEFF"
                      fill="#CABEFF"
                    />
                  </View>
                  <Text className="text-[17px] text-foreground">{item.folder.name}</Text>
                </View>
                {isEditing ? (
                  <GripVertical size={20} color="#8E8C99" />
                ) : (
                  <View className="flex-row items-center gap-1.5">
                    <Text className="text-[15px] text-muted-foreground">
                      {getFolderCount(item.folder.id)}
                    </Text>
                    <ChevronRight size={16} color="#8E8C99" />
                  </View>
                )}
              </Pressable>
            </SwipeableListItem>
          </View>
        )
      }

      if (item.type === "trash") {
        return (
          <View
            className={`mt-5 overflow-hidden rounded-3xl bg-white/7 ${isEditing ? "opacity-40" : ""}`}
          >
            <Pressable
              disabled={isEditing}
              onPress={() => router.push("/folders/trash" as const)}
              className="flex-row items-center justify-between px-4 py-3.5 active:bg-white/12"
            >
              <View className="flex-row items-center gap-3">
                <View className="size-8 items-center justify-center rounded-lg">
                  <Trash2 size={18} color="#FF6B6B" />
                </View>
                <Text className="text-[17px] text-[#FF6B6B]">Trash</Text>
              </View>
              <View className="flex-row items-center gap-1.5">
                <Text className="text-[15px] text-muted-foreground">{counts.trash}</Text>
                <ChevronRight size={16} color="#8E8C99" />
              </View>
            </Pressable>
          </View>
        )
      }

      return null
    },
    [
      counts.trash,
      getFolderCount,
      handleDeleteFolder,
      isEditing,
      router,
      selectedFolderIds,
      toggleSelectFolder,
    ],
  )

  return (
    <View className="flex-1 bg-background">
      <Stack.Screen
        options={{
          title: "Folders",
          headerLargeTitle: true,
          headerShown: true,
          headerRight: () =>
            isEditing ? (
              <Pressable onPress={toggleEditMode} hitSlop={8} className="active:opacity-60">
                <Text className="text-[17px] font-semibold text-primary">Done</Text>
              </Pressable>
            ) : (
              <View className="flex-row items-center gap-5">
                {foldersList.length > 0 && (
                  <Pressable onPress={toggleEditMode} hitSlop={8} className="active:opacity-60">
                    <Text className="text-[17px] font-medium text-white">Edit</Text>
                  </Pressable>
                )}
                <Pressable
                  onPress={() => {
                    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
                    sheetRef.current?.open()
                  }}
                  hitSlop={8}
                  className="active:opacity-60"
                >
                  <Plus size={22} color="#ffffff" />
                </Pressable>
              </View>
            ),
        }}
      />

      <FlatList
        data={listData}
        keyExtractor={(item) => item.id}
        renderItem={renderItem}
        className="flex-1"
        contentInsetAdjustmentBehavior="automatic"
        keyboardShouldPersistTaps="handled"
        contentContainerStyle={{
          paddingHorizontal: 20,
          paddingTop: 12,
          paddingBottom: 110,
        }}
      />

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
