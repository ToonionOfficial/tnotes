import * as Haptics from "expo-haptics"
import { Stack, useRouter } from "expo-router"
import { Trash2 } from "lucide-react-native"
import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import {
  ActivityIndicator,
  Alert,
  type NativeScrollEvent,
  type NativeSyntheticEvent,
  Pressable,
  ScrollView,
  Text,
  useWindowDimensions,
  View,
} from "react-native"
import { Drawer } from "react-native-drawer-layout"
import { useSafeAreaInsets } from "react-native-safe-area-context"
import { BottomBar } from "@/components/BottomBar"
import { DraggableFolderSection } from "@/components/DraggableFolderSection"
import { FolderActionSheet, type FolderActionSheetRef } from "@/components/FolderActionSheet"
import { FolderIcon } from "@/components/FolderIcon"
import { HomeHeader } from "@/components/HomeHeader"
import { NewFolderSheet, type NewFolderSheetRef } from "@/components/NewFolderForm"
import { SideDrawerContent } from "@/components/SideDrawerContent"
import { VirtualFolderCard } from "@/components/VirtualFolderCard"
import type { Folder } from "@/db/schema"
import {
  useBatchDeleteFolders,
  useDeleteFolder,
  useInfiniteFolders,
  useReorderFolders,
} from "@/hooks/useFolders"
import { useFolderNoteCounts } from "@/hooks/useNotes"
import { useSyncState } from "@/hooks/useSyncState"

export default function FoldersScreen() {
  const router = useRouter()
  const insets = useSafeAreaInsets()
  const { width } = useWindowDimensions()
  const {
    data: folderPages,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteFolders({ parentId: null })
  const foldersList = useMemo(
    () => folderPages?.pages.flatMap((page) => page.folders) ?? [],
    [folderPages],
  )
  const deleteFolder = useDeleteFolder()
  const batchDeleteFolders = useBatchDeleteFolders()
  const reorderFolders = useReorderFolders()
  const { data: counts = { total: 0, byFolder: {}, trash: 0 } } = useFolderNoteCounts()
  const { data: syncStatus } = useSyncState()

  const [folders, setFolders] = useState<Folder[]>(foldersList)
  useEffect(() => {
    setFolders(foldersList)
  }, [foldersList])

  const sheetRef = useRef<NewFolderSheetRef>(null)
  const folderActionSheetRef = useRef<FolderActionSheetRef>(null)
  const [isDrawerOpen, setIsDrawerOpen] = useState(false)
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

  const handleSelectAllFolders = useCallback(() => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    setSelectedFolderIds(new Set(folders.map((f) => f.id)))
  }, [folders])

  const handleDeselectAllFolders = useCallback(() => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    setSelectedFolderIds(new Set())
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

  const handleDeleteFolder = useCallback(
    (folder: Folder) => {
      void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
      Alert.alert(`Delete "${folder.name}"?`, "All notes inside will be moved to the Trash.", [
        { text: "Cancel", style: "cancel" },
        {
          text: "Delete",
          style: "destructive",
          onPress: () => deleteFolder.mutate(folder.id),
        },
      ])
    },
    [deleteFolder],
  )

  const handleLongPressFolder = useCallback((folder: Folder) => {
    folderActionSheetRef.current?.open(folder)
  }, [])

  const handleEditFolderFromAction = useCallback((folder: Folder) => {
    sheetRef.current?.open(folder)
  }, [])

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

  const handleScroll = useCallback(
    (event: NativeSyntheticEvent<NativeScrollEvent>) => {
      const { layoutMeasurement, contentOffset, contentSize } = event.nativeEvent
      const paddingToBottom = 300
      if (layoutMeasurement.height + contentOffset.y >= contentSize.height - paddingToBottom) {
        if (hasNextPage && !isFetchingNextPage) {
          fetchNextPage()
        }
      }
    },
    [fetchNextPage, hasNextPage, isFetchingNextPage],
  )

  const getFolderCount = useCallback(
    (folderId: string | null) => {
      if (folderId === null) {
        return counts.total
      }
      return counts.byFolder[folderId] ?? 0
    },
    [counts],
  )

  const isAllSelected = selectedFolderIds.size === folders.length && folders.length > 0
  const isSelected = selectedFolderIds.size > 0

  return (
    <Drawer
      open={isDrawerOpen}
      onOpen={() => {
        void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
        setIsDrawerOpen(true)
      }}
      onClose={() => {
        void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
        setIsDrawerOpen(false)
      }}
      drawerType="back"
      drawerStyle={{
        width: "80%",
        maxWidth: 320,
        backgroundColor: "#141318",
      }}
      swipeEdgeWidth={width}
      swipeMinDistance={30}
      swipeEnabled={!isEditing}
      renderDrawerContent={() => (
        <SideDrawerContent
          trashCount={counts.trash}
          username={syncStatus?.username}
          isConnected={syncStatus?.isConnected}
          onPressTrash={() => {
            setIsDrawerOpen(false)
            router.push("/folders/trash" as const)
          }}
          onPressFavorites={() => {
            setIsDrawerOpen(false)
            router.push("/folders/all" as const)
          }}
          onPressProfile={() => {
            setIsDrawerOpen(false)
            router.push("/settings" as const)
          }}
          onPressSettings={() => {
            setIsDrawerOpen(false)
            router.push("/settings" as const)
          }}
        />
      )}
    >
      <View className="flex-1 bg-background">
        <Stack.Screen options={{ headerShown: false }} />

        {/* Top Navigation & Title Bar */}
        <View style={{ paddingTop: insets.top + 8 }} className="px-5">
          <HomeHeader
            isEditing={isEditing}
            hasFolders={foldersList.length > 0}
            selectedCount={selectedFolderIds.size}
            onPressMenu={() => setIsDrawerOpen(true)}
            onToggleEdit={toggleEditMode}
            onPressNewFolder={() => {
              sheetRef.current?.open()
            }}
            onDeleteSelected={handleBatchDeleteFolders}
          />

          {/* Large Screen Title */}
          <Text className="mb-1 mt-2 text-[34px] font-bold tracking-tight text-white">Folders</Text>
        </View>

        <ScrollView
          className="flex-1"
          contentInsetAdjustmentBehavior="automatic"
          keyboardShouldPersistTaps="handled"
          onScroll={handleScroll}
          scrollEventThrottle={100}
          contentContainerStyle={{
            paddingHorizontal: 20,
            paddingTop: 6,
            paddingBottom: 110,
          }}
        >
          <VirtualFolderCard
            title="All Notes"
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
            onLongPressFolder={handleLongPressFolder}
            onDeleteFolder={handleDeleteFolder}
            onReorder={handleReorder}
          />

          {isFetchingNextPage && (
            <View className="py-4 items-center justify-center">
              <ActivityIndicator size="small" color="#CABEFF" />
            </View>
          )}
        </ScrollView>

        {isEditing ? (
          <View
            style={{
              paddingBottom: Math.max(insets.bottom, 16),
            }}
            className="absolute bottom-0 left-0 right-0 border-t border-white/10 bg-[#1C1B20]/95 px-6 pt-3 backdrop-blur-xl"
          >
            <View className="flex-row items-center justify-between">
              {/* Select All / Deselect All */}
              <Pressable
                onPress={isAllSelected ? handleDeselectAllFolders : handleSelectAllFolders}
                hitSlop={8}
                className="rounded-full bg-white/[0.08] px-3.5 py-1.5 active:opacity-60"
              >
                <Text className="text-[13px] font-medium text-white/80">
                  {isAllSelected ? "Deselect All" : "Select All"}
                </Text>
              </Pressable>

              {/* Delete Action */}
              <Pressable
                onPress={() => {
                  if (isSelected) {
                    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
                    handleBatchDeleteFolders()
                  }
                }}
                disabled={!isSelected}
                hitSlop={8}
                className={`flex-row items-center gap-2 py-2 ${
                  isSelected ? "active:opacity-60" : "opacity-35"
                }`}
              >
                <Text
                  className={`text-[15px] font-medium ${
                    isSelected ? "text-[#FF6B6B]" : "text-muted-foreground"
                  }`}
                >
                  Delete
                  {selectedFolderIds.size > 0 ? ` (${selectedFolderIds.size})` : ""}
                </Text>
                <Trash2 size={20} color={isSelected ? "#FF6B6B" : "#8E8C99"} />
              </Pressable>
            </View>
          </View>
        ) : (
          <BottomBar
            searchValue={searchValue}
            onSearchChange={setSearchValue}
            onPressNewNote={() => router.push("/notes/new" as const)}
          />
        )}

        <NewFolderSheet
          ref={sheetRef}
          onCreated={(folderId) => router.push(`/folders/${folderId}` as const)}
        />

        <FolderActionSheet
          ref={folderActionSheetRef}
          onEdit={handleEditFolderFromAction}
          onDelete={handleDeleteFolder}
          getFolderCount={getFolderCount}
        />
      </View>
    </Drawer>
  )
}
