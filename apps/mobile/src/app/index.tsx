import { LegendList } from "@legendapp/list/react-native"
import * as Haptics from "expo-haptics"
import { Stack, useRouter } from "expo-router"
import { Trash2 } from "lucide-react-native"
import { useCallback, useMemo, useRef, useState } from "react"
import { ActivityIndicator, Alert, Pressable, Text, useWindowDimensions, View } from "react-native"
import { Drawer } from "react-native-drawer-layout"
import { useSafeAreaInsets } from "react-native-safe-area-context"
import { BottomBar } from "@/components/BottomBar"
import { DraggableFolderSection } from "@/components/DraggableFolderSection"
import { FolderActionSheet, type FolderActionSheetRef } from "@/components/FolderActionSheet"
import { FolderIcon } from "@/components/FolderIcon"
import { FolderRow } from "@/components/FolderRow"
import { HomeHeader } from "@/components/HomeHeader"
import { NewFolderSheet, type NewFolderSheetRef } from "@/components/NewFolderForm"
import { SideDrawerContent } from "@/components/SideDrawerContent"
import { VirtualFolderCard } from "@/components/VirtualFolderCard"
import { moveIdInList } from "@/db/queries"
import type { Folder } from "@/db/schema"
import { useAppTheme } from "@/hooks/useAppTheme"
import {
  useBatchDeleteFolders,
  useDeleteFolder,
  useFrequentFolders,
  useInfiniteFolders,
  useReorderFrequentFolders,
} from "@/hooks/useFolders"
import { useFolderNoteCounts } from "@/hooks/useNotes"
import { useSyncState } from "@/hooks/useSyncState"

export default function FoldersScreen() {
  const router = useRouter()
  const insets = useSafeAreaInsets()
  const { width } = useWindowDimensions()
  const { colors } = useAppTheme()
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
  const reorderFrequent = useReorderFrequentFolders()
  const { data: counts = { total: 0, byFolder: {}, trash: 0 } } = useFolderNoteCounts()
  const { data: syncStatus } = useSyncState()
  const { data: frequentFolders = [] } = useFrequentFolders()

  // The main list stays static (never draggable): manual drag-reorder of
  // 100s of rows is what killed the app, and ordering lives in the
  // 5-row "Frequently Used" section instead.
  const frequentIds = useMemo(
    () => new Set(frequentFolders.map((folder) => folder.id)),
    [frequentFolders],
  )
  const mainFolders = useMemo(
    () => foldersList.filter((folder) => !frequentIds.has(folder.id)),
    [foldersList, frequentIds],
  )

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
    setSelectedFolderIds(new Set(foldersList.map((f) => f.id)))
  }, [foldersList])

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

  const handleReorderFrequent = useCallback(
    (fromIndex: number, toIndex: number) => {
      // Persist only the device-local order list — the optimistic order
      // comes from useReorderFrequentFolders' onMutate cache patch.
      const moved = frequentFolders[fromIndex]
      if (!moved || fromIndex === toIndex) return
      void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
      reorderFrequent.mutate(
        moveIdInList(
          frequentFolders.map((folder) => folder.id),
          moved.id,
          toIndex,
        ),
      )
    },
    [frequentFolders, reorderFrequent],
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

  const handlePressFolder = useCallback(
    (folderId: string) => router.push(`/folders/${folderId}` as const),
    [router],
  )

  const renderFolderRow = useCallback(
    ({ item, index }: { item: Folder; index: number }) => (
      <FolderRow
        folder={item}
        isFirst={index === 0}
        isLast={index === mainFolders.length - 1}
        isOnly={mainFolders.length === 1}
        isEditing={isEditing}
        isSelected={selectedFolderIds.has(item.id)}
        noteCount={getFolderCount(item.id)}
        onToggleSelect={toggleSelectFolder}
        onPressFolder={handlePressFolder}
        onLongPressFolder={handleLongPressFolder}
        onDeleteFolder={handleDeleteFolder}
      />
    ),
    [
      mainFolders.length,
      isEditing,
      selectedFolderIds,
      getFolderCount,
      toggleSelectFolder,
      handlePressFolder,
      handleLongPressFolder,
      handleDeleteFolder,
    ],
  )

  const folderKeyExtractor = useCallback((item: Folder) => item.id, [])

  const legendListExtraData = useMemo(
    () => ({
      isEditing,
      selectedFolderIds,
      folderCount: mainFolders.length,
      counts,
    }),
    [isEditing, selectedFolderIds, mainFolders.length, counts],
  )

  const isAllSelected = selectedFolderIds.size === foldersList.length && foldersList.length > 0
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
        backgroundColor: colors.background,
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
            onPressMenu={() => setIsDrawerOpen(true)}
            onToggleEdit={toggleEditMode}
            onPressNewFolder={() => {
              sheetRef.current?.open()
            }}
          />

          {/* Large Screen Title */}
          <Text className="mb-1 mt-2 text-[34px] font-bold tracking-tight text-foreground">
            Folders
          </Text>
        </View>

        <LegendList
          className="flex-1"
          data={mainFolders}
          renderItem={renderFolderRow}
          keyExtractor={folderKeyExtractor}
          extraData={legendListExtraData}
          contentInsetAdjustmentBehavior="automatic"
          keyboardShouldPersistTaps="handled"
          contentContainerStyle={{
            paddingHorizontal: 20,
            paddingTop: 6,
            paddingBottom: 110,
          }}
          ListHeaderComponent={
            <>
              <VirtualFolderCard
                title="All Notes"
                icon={
                  <FolderIcon
                    name="folder"
                    size={20}
                    color={colors.primary}
                    fill={colors.primary}
                  />
                }
                count={getFolderCount(null)}
                isEditing={isEditing}
                onPress={() => router.push("/folders/all" as const)}
              />

              {frequentFolders.length > 0 && (
                <View>
                  <View className="mb-1 mt-2 flex-row items-center justify-between px-1">
                    <Text className="text-[13px] font-semibold tracking-wider text-muted-foreground/60 uppercase">
                      Frequently Used
                    </Text>
                    <Text className="text-[13px] text-muted-foreground/60">
                      {frequentFolders.length} {frequentFolders.length === 1 ? "folder" : "folders"}
                    </Text>
                  </View>
                  <DraggableFolderSection
                    folders={frequentFolders}
                    isEditing={isEditing}
                    selectedFolderIds={selectedFolderIds}
                    getFolderCount={getFolderCount}
                    onToggleSelect={toggleSelectFolder}
                    onPressFolder={handlePressFolder}
                    onLongPressFolder={handleLongPressFolder}
                    onDeleteFolder={handleDeleteFolder}
                    onReorder={handleReorderFrequent}
                  />
                </View>
              )}
            </>
          }
          onEndReached={() => {
            if (hasNextPage && !isFetchingNextPage) {
              fetchNextPage()
            }
          }}
          onEndReachedThreshold={0.5}
          ListFooterComponent={
            isFetchingNextPage ? (
              <View className="py-4 items-center justify-center">
                <ActivityIndicator size="small" color="#CABEFF" />
              </View>
            ) : null
          }
        />

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
                className="rounded-full bg-white/8 px-3.5 py-1.5 active:opacity-60"
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
