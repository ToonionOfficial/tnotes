import * as Haptics from "expo-haptics"
import { Stack, useRouter } from "expo-router"
import { ChevronRight, Circle, CircleCheck, GripVertical, Plus, Trash2 } from "lucide-react-native"
import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react"
import { Alert, FlatList, Platform, Pressable, Text, View } from "react-native"
import Animated, {
  Easing,
  interpolate,
  useAnimatedStyle,
  useSharedValue,
  withTiming,
} from "react-native-reanimated"
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

interface FolderRowProps {
  folder: Folder
  isFirst: boolean
  isLast: boolean
  isOnly: boolean
  isEditing: boolean
  isSelected: boolean
  noteCount: number
  onPress: () => void
  onDelete: () => void
}

const FolderRow = memo(function FolderRow({
  folder,
  isFirst,
  isLast,
  isOnly,
  isEditing,
  isSelected,
  noteCount,
  onPress,
  onDelete,
}: FolderRowProps) {
  const editProgress = useSharedValue(isEditing ? 1 : 0)

  useEffect(() => {
    editProgress.value = withTiming(isEditing ? 1 : 0, {
      duration: 250,
      easing: Easing.bezier(0.25, 0.1, 0.25, 1),
    })
  }, [isEditing, editProgress])

  const selectCircleStyle = useAnimatedStyle(() => ({
    width: interpolate(editProgress.value, [0, 1], [0, 32]),
    opacity: editProgress.value,
    transform: [
      { scale: interpolate(editProgress.value, [0, 1], [0.5, 1]) },
      { translateX: interpolate(editProgress.value, [0, 1], [-12, 0]) },
    ],
  }))

  const handleStyle = useAnimatedStyle(() => ({
    opacity: editProgress.value,
    transform: [{ scale: interpolate(editProgress.value, [0, 1], [0.6, 1]) }],
  }))

  const countStyle = useAnimatedStyle(() => ({
    opacity: interpolate(editProgress.value, [0, 1], [1, 0]),
    transform: [{ scale: interpolate(editProgress.value, [0, 1], [1, 0.8]) }],
  }))

  const roundingClass = isOnly
    ? "rounded-3xl"
    : isFirst
      ? "rounded-t-3xl"
      : isLast
        ? "rounded-b-3xl"
        : ""

  return (
    <View className={`overflow-hidden bg-white/7 ${roundingClass} ${isFirst ? "mt-5" : ""}`}>
      {!isFirst && <View className="ml-15 h-[0.5px] bg-white/8" />}
      <SwipeableListItem
        enabled={!isEditing}
        rounded={isOnly ? "only" : isFirst ? "first" : isLast ? "last" : "middle"}
        rightAction={{
          label: "Delete",
          color: "#D94C5C",
          icon: <Trash2 size={20} color="#FFFFFF" />,
          onPress: onDelete,
        }}
      >
        <Pressable
          onPress={onPress}
          className={`flex-row items-center justify-between px-4 py-3.5 active:bg-white/12 ${roundingClass}`}
        >
          <View className="flex-1 flex-row items-center">
            <Animated.View style={selectCircleStyle} className="justify-center overflow-hidden">
              {isSelected ? (
                <CircleCheck size={22} color="#CABEFF" fill="#CABEFF" />
              ) : (
                <Circle size={22} color="#8E8C99" />
              )}
            </Animated.View>
            <View className="size-8 items-center justify-center rounded-lg">
              <FolderIcon
                name={folder.icon || DEFAULT_FOLDER_ICON}
                size={20}
                color="#CABEFF"
                fill="#CABEFF"
              />
            </View>
            <Text className="ml-3 flex-1 text-[17px] text-foreground" numberOfLines={1}>
              {folder.name}
            </Text>
          </View>

          <View className="items-end justify-center">
            <Animated.View
              style={[handleStyle, { position: "absolute", right: 0 }]}
              pointerEvents={isEditing ? "auto" : "none"}
            >
              <GripVertical size={20} color="#8E8C99" />
            </Animated.View>
            <Animated.View style={countStyle} pointerEvents={isEditing ? "none" : "auto"}>
              <View className="flex-row items-center gap-1.5">
                <Text className="text-[15px] text-muted-foreground">{noteCount}</Text>
                <ChevronRight size={16} color="#8E8C99" />
              </View>
            </Animated.View>
          </View>
        </Pressable>
      </SwipeableListItem>
    </View>
  )
})

const VirtualFolderCard = memo(function VirtualFolderCard({
  title,
  icon,
  count,
  isEditing,
  onPress,
  className = "",
  textColor = "text-foreground",
}: {
  title: string
  icon: React.ReactNode
  count: number
  isEditing: boolean
  onPress: () => void
  className?: string
  textColor?: string
}) {
  const animatedStyle = useAnimatedStyle(() => ({
    opacity: withTiming(isEditing ? 0.4 : 1, { duration: 250 }),
  }))

  return (
    <Animated.View
      style={animatedStyle}
      className={`overflow-hidden rounded-3xl bg-white/7 ${className}`}
    >
      <Pressable
        disabled={isEditing}
        onPress={onPress}
        className="flex-row items-center justify-between px-4 py-3.5 active:bg-white/12"
      >
        <View className="flex-row items-center gap-3">
          <View className="size-8 items-center justify-center rounded-lg">{icon}</View>
          <Text className={`text-[17px] ${textColor}`}>{title}</Text>
        </View>
        <View className="flex-row items-center gap-1.5">
          <Text className="text-[15px] text-muted-foreground">{count}</Text>
          <ChevronRight size={16} color="#8E8C99" />
        </View>
      </Pressable>
    </Animated.View>
  )
})

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
          <VirtualFolderCard
            title="Notes"
            icon={<FolderIcon name="folder" size={20} color="#CABEFF" fill="#CABEFF" />}
            count={getFolderCount(null)}
            isEditing={isEditing}
            onPress={() => router.push("/folders/all" as const)}
          />
        )
      }

      if (item.type === "folder") {
        const isOnly = item.isFirst && item.isLast
        return (
          <FolderRow
            folder={item.folder}
            isFirst={item.isFirst}
            isLast={item.isLast}
            isOnly={isOnly}
            isEditing={isEditing}
            isSelected={selectedFolderIds.has(item.folder.id)}
            noteCount={getFolderCount(item.folder.id)}
            onPress={() => {
              if (isEditing) {
                toggleSelectFolder(item.folder.id)
              } else {
                router.push(`/folders/${item.folder.id}` as const)
              }
            }}
            onDelete={() => handleDeleteFolder(item.folder)}
          />
        )
      }

      if (item.type === "trash") {
        return (
          <VirtualFolderCard
            title="Trash"
            icon={<Trash2 size={18} color="#FF6B6B" />}
            count={counts.trash}
            isEditing={isEditing}
            className="mt-5"
            textColor="text-[#FF6B6B]"
            onPress={() => router.push("/folders/trash" as const)}
          />
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
          unstable_headerRightItems: () =>
            isEditing
              ? [
                  {
                    type: "button",
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
                    <Pressable onPress={toggleEditMode} hitSlop={8} className="active:opacity-60">
                      <Text className="text-[17px] font-semibold text-white">Done</Text>
                    </Pressable>
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
                          <View className="mx-1 h-3.5 w-[1px] bg-white/20" />
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
