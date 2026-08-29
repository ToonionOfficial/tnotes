import * as Haptics from "expo-haptics"
import { Stack, useRouter } from "expo-router"
import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import {
  ActivityIndicator,
  Alert,
  type NativeScrollEvent,
  type NativeSyntheticEvent,
  ScrollView,
  Text,
  View,
} from "react-native"
import { Drawer } from "react-native-drawer-layout"
import { useSafeAreaInsets } from "react-native-safe-area-context"
import { BottomBar } from "@/components/BottomBar"
import { DraggableFolderSection } from "@/components/DraggableFolderSection"
import { FolderIcon } from "@/components/FolderIcon"
import { HomeHeader } from "@/components/HomeHeader"
import { NewFolderSheet, type NewFolderSheetRef } from "@/components/NewFolderForm"
import { PerformanceBenchmark } from "@/components/PerformanceBenchmark"
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

export default function FoldersScreen() {
  const router = useRouter()
  const { data: folderPages, fetchNextPage, hasNextPage, isFetchingNextPage } = useInfiniteFolders()
  const foldersList = useMemo(
    () => folderPages?.pages.flatMap((page) => page.folders) ?? [],
    [folderPages],
  )
  const deleteFolder = useDeleteFolder()
  const batchDeleteFolders = useBatchDeleteFolders()
  const reorderFolders = useReorderFolders()
  const { data: counts = { total: 0, byFolder: {}, trash: 0 } } = useFolderNoteCounts()

  const [folders, setFolders] = useState<Folder[]>(foldersList)
  useEffect(() => {
    setFolders(foldersList)
  }, [foldersList])

  const sheetRef = useRef<NewFolderSheetRef>(null)
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

  const handleScroll = useCallback(
    (event: NativeSyntheticEvent<NativeScrollEvent>) => {
      const { layoutMeasurement, contentOffset, contentSize } = event.nativeEvent
      const paddingToBottom = 300
      if (layoutMeasurement.height + contentOffset.y >= contentSize.height - paddingToBottom) {
        if (hasNextPage && !isFetchingNextPage) {
          void fetchNextPage()
        }
      }
    },
    [hasNextPage, isFetchingNextPage, fetchNextPage],
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

  const insets = useSafeAreaInsets()

  return (
    <Drawer
      open={isDrawerOpen}
      onOpen={() => setIsDrawerOpen(true)}
      onClose={() => setIsDrawerOpen(false)}
      drawerType="back"
      drawerStyle={{ width: "80%", maxWidth: 320, backgroundColor: "#141318" }}
      swipeEnabled={!isEditing}
      renderDrawerContent={() => (
        <SideDrawerContent
          trashCount={counts.trash}
          onPressTrash={() => {
            setIsDrawerOpen(false)
            router.push("/folders/trash" as const)
          }}
          onPressFavorites={() => {
            setIsDrawerOpen(false)
            router.push("/folders/all" as const)
          }}
        />
      )}
    >
      <View className="flex-1 bg-background">
        <Stack.Screen options={{ headerShown: false }} />

        {/* Top Navigation & Title Bar (Slides with entire screen) */}
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

          {isFetchingNextPage && (
            <View className="py-4 items-center justify-center">
              <ActivityIndicator size="small" color="#CABEFF" />
            </View>
          )}

          <PerformanceBenchmark />
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
    </Drawer>
  )
}
