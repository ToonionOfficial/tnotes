import { LegendList } from "@legendapp/list/react-native"
import * as Haptics from "expo-haptics"
import { Stack, useLocalSearchParams, useRouter } from "expo-router"
import { ChevronLeft, FolderPlus } from "lucide-react-native"
import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import {
  ActivityIndicator,
  Alert,
  InteractionManager,
  Platform,
  Pressable,
  Text,
  View,
} from "react-native"
import { BottomBar } from "@/components/BottomBar"
import { DraggableFolderSection } from "@/components/DraggableFolderSection"
import { FolderActionSheet, type FolderActionSheetRef } from "@/components/FolderActionSheet"
import { MoveNoteSheet, type MoveNoteSheetRef } from "@/components/MoveNoteSheet"
import { NewFolderSheet, type NewFolderSheetRef } from "@/components/NewFolderForm"
import { NoteActionSheet, type NoteActionSheetRef } from "@/components/NoteActionSheet"
import { NoteListItem } from "@/components/NoteListItem"
import NoteSectionHeader from "@/components/NoteSectionHeader"
import { NotesEditBottomBar } from "@/components/NotesEditBottomBar"
import type { SearchResult } from "@/db/queries"
import type { Note } from "@/db/schema"
import { useDebouncedValue } from "@/hooks/useDebouncedValue"
import {
  useDeleteFolder,
  useFolder,
  useInfiniteFolders,
  useReorderFolders,
} from "@/hooks/useFolders"
import {
  useBatchDeleteNotesPermanently,
  useBatchMoveNotes,
  useBatchTrashNotes,
  useDeleteNotePermanently,
  useFolderNoteCounts,
  useNotes,
  useRestoreNote,
  useTogglePinNote,
  useTrashNote,
  useUpdateNote,
} from "@/hooks/useNotes"
import { groupNotesByDate } from "@/utils/date"

type FlatNoteItem =
  | { type: "header"; id: string; title: string }
  | {
      type: "note"
      id: string
      item: Note | SearchResult
      isFirst: boolean
      isLast: boolean
    }

export default function FolderNotesScreen() {
  const router = useRouter()
  const { id } = useLocalSearchParams<{ id: string }>()

  const isAll = id === "all" || !id
  const isTrash = id === "trash"
  const folderId = isAll || isTrash ? undefined : id

  const [searchValue, setSearchValue] = useState("")
  const debouncedSearch = useDebouncedValue(searchValue, 150)
  const [isEditing, setIsEditing] = useState(false)
  const [selectedNoteIds, setSelectedNoteIds] = useState<Set<string>>(new Set())
  const [isReadyToLoad, setIsReadyToLoad] = useState(false)
  const [actionNote, setActionNote] = useState<Note | SearchResult | null>(null)

  const actionSheetRef = useRef<NoteActionSheetRef>(null)
  const moveSheetRef = useRef<MoveNoteSheetRef>(null)
  const newFolderSheetRef = useRef<NewFolderSheetRef>(null)
  const folderActionSheetRef = useRef<FolderActionSheetRef>(null)

  useEffect(() => {
    const interaction = InteractionManager.runAfterInteractions(() => setIsReadyToLoad(true))
    return () => interaction.cancel()
  }, [])

  const { data: folder } = useFolder(folderId ?? null, isReadyToLoad)
  const title = isAll ? "Notes" : isTrash ? "Trash" : folder?.name || "Notes"

  const {
    data: notePages,
    isLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useNotes(
    {
      search: debouncedSearch,
      folderId: isAll || isTrash ? undefined : folderId,
      trashed: Boolean(isTrash),
    },
    isReadyToLoad,
  )
  const notesList = useMemo(() => notePages?.pages.flatMap((page) => page.notes) ?? [], [notePages])

  const { data: subfolderPages } = useInfiniteFolders(
    { parentId: folderId ?? null },
    isReadyToLoad && Boolean(folderId),
  )
  const subfoldersList = useMemo(
    () => (folderId ? (subfolderPages?.pages.flatMap((page) => page.folders) ?? []) : []),
    [subfolderPages, folderId],
  )

  const { data: counts = { total: 0, byFolder: {}, trash: 0 } } = useFolderNoteCounts()
  const deleteFolderMutation = useDeleteFolder()
  const reorderFoldersMutation = useReorderFolders()

  const togglePinNote = useTogglePinNote()
  const updateNoteMutation = useUpdateNote()
  const batchMoveNotes = useBatchMoveNotes()
  const trashNote = useTrashNote()
  const restoreNote = useRestoreNote()
  const deleteNotePermanently = useDeleteNotePermanently()
  const batchTrashNotes = useBatchTrashNotes()
  const batchDeleteNotesPermanently = useBatchDeleteNotesPermanently()

  const toggleSelectNote = useCallback((noteId: string) => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    setSelectedNoteIds((prev) => {
      const next = new Set(prev)
      if (next.has(noteId)) {
        next.delete(noteId)
      } else {
        next.add(noteId)
      }
      return next
    })
  }, [])

  const handleSelectAll = useCallback(() => {
    setSelectedNoteIds(new Set(notesList.map((n) => n.id)))
  }, [notesList])

  const handleDeselectAll = useCallback(() => {
    setSelectedNoteIds(new Set())
  }, [])

  const toggleEditMode = useCallback(() => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
    setIsEditing((prev) => {
      if (prev) {
        setSelectedNoteIds(new Set())
      }
      return !prev
    })
  }, [])

  const handleBatchDeleteNotes = useCallback(() => {
    const ids = Array.from(selectedNoteIds)
    if (ids.length === 0) return

    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
    const count = ids.length

    if (isTrash) {
      Alert.alert(
        `Delete ${count} ${count === 1 ? "Note" : "Notes"} Permanently?`,
        "This action cannot be undone.",
        [
          { text: "Cancel", style: "cancel" },
          {
            text: `Delete ${count === 1 ? "Note" : "Notes"}`,
            style: "destructive",
            onPress: () => {
              batchDeleteNotesPermanently.mutate(ids)
              setSelectedNoteIds(new Set())
            },
          },
        ],
      )
    } else {
      Alert.alert(
        `Move ${count} ${count === 1 ? "Note" : "Notes"} to Trash?`,
        "You can restore them later from the Trash.",
        [
          { text: "Cancel", style: "cancel" },
          {
            text: `Move to Trash`,
            style: "destructive",
            onPress: () => {
              batchTrashNotes.mutate(ids)
              setSelectedNoteIds(new Set())
            },
          },
        ],
      )
    }
  }, [batchDeleteNotesPermanently, batchTrashNotes, isTrash, selectedNoteIds])

  const handleTogglePin = useCallback(
    (noteId: string) => {
      void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
      togglePinNote.mutate(noteId)
    },
    [togglePinNote],
  )

  const handleRestore = useCallback(
    (noteId: string) => {
      void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
      restoreNote.mutate(noteId)
    },
    [restoreNote],
  )

  const handleDelete = useCallback(
    (noteId: string) => {
      void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
      if (isTrash) {
        Alert.alert("Delete Note Permanently?", "This action cannot be undone.", [
          { text: "Cancel", style: "cancel" },
          {
            text: "Delete",
            style: "destructive",
            onPress: () => deleteNotePermanently.mutate(noteId),
          },
        ])
      } else {
        trashNote.mutate(noteId)
      }
    },
    [deleteNotePermanently, isTrash, trashNote],
  )

  const handleLongPressNote = useCallback((note: Note | SearchResult) => {
    setActionNote(note)
    actionSheetRef.current?.open(note)
  }, [])

  const handleActionMoveToFolder = useCallback((note: Note | SearchResult) => {
    moveSheetRef.current?.open(note)
  }, [])

  const handleSelectTargetFolder = useCallback(
    (targetFolderId: string | null) => {
      if (isEditing && selectedNoteIds.size > 0) {
        batchMoveNotes.mutate({
          ids: Array.from(selectedNoteIds),
          folderId: targetFolderId,
        })
        setSelectedNoteIds(new Set())
        setIsEditing(false)
      } else if (actionNote) {
        updateNoteMutation.mutate({
          id: actionNote.id,
          input: { folderId: targetFolderId },
        })
      }
    },
    [actionNote, batchMoveNotes, isEditing, selectedNoteIds, updateNoteMutation],
  )

  const flatItems = useMemo(() => {
    if (notesList.length === 0) return []

    const sections = groupNotesByDate(notesList, {
      ignorePinned: Boolean(isTrash),
    })
    const items: FlatNoteItem[] = []

    for (const section of sections) {
      items.push({
        type: "header",
        id: `header-${section.title}`,
        title: section.title,
      })

      const len = section.data.length
      section.data.forEach((note, index) => {
        items.push({
          type: "note",
          id: note.id,
          item: note,
          isFirst: index === 0,
          isLast: index === len - 1,
        })
      })
    }

    return items
  }, [notesList, isTrash])

  const noteCount = notesList.length

  const handlePressNote = useCallback(
    (noteId: string) => {
      router.push(`/notes/${noteId}` as const)
    },
    [router],
  )

  const handleNotePress = useCallback(
    (noteId: string) => {
      if (isEditing) {
        toggleSelectNote(noteId)
      } else {
        handlePressNote(noteId)
      }
    },
    [handlePressNote, isEditing, toggleSelectNote],
  )

  const handlePressNewNote = () => {
    if (isTrash || !folderId) {
      router.push("/notes/new" as const)
    } else {
      router.push({
        pathname: "/notes/[id]",
        params: { id: "new", folderId },
      })
    }
  }

  const renderItem = useCallback(
    ({ item }: { item: FlatNoteItem }) => {
      if (item.type === "header") {
        return <NoteSectionHeader title={item.title} />
      }

      return (
        <NoteListItem
          item={item.item}
          isFirst={item.isFirst}
          isLast={item.isLast}
          isTrash={Boolean(isTrash)}
          isEditing={isEditing}
          isSelected={selectedNoteIds.has(item.item.id)}
          onPress={handleNotePress}
          onLongPress={handleLongPressNote}
          onTogglePin={isTrash ? undefined : handleTogglePin}
          onRestore={isTrash ? handleRestore : undefined}
          onDelete={handleDelete}
        />
      )
    },
    [
      handleDelete,
      handleRestore,
      handleTogglePin,
      handleLongPressNote,
      handleNotePress,
      isEditing,
      isTrash,
      selectedNoteIds,
    ],
  )

  const keyExtractor = useCallback((item: FlatNoteItem) => item.id, [])

  const legendListExtraData = useMemo(
    () => ({
      isEditing,
      selectedCount: selectedNoteIds.size,
      selectedNoteIds,
    }),
    [isEditing, selectedNoteIds],
  )

  return (
    <View className="flex-1 bg-background">
      <Stack.Screen
        options={{
          title,
          headerLargeTitle: true,
          headerShown: true,
          headerBackVisible: false,
          unstable_headerLeftItems: () => [
            {
              type: "button",
              label: "Back",
              icon: {
                name: "chevron.left",
                type: "sfSymbol",
              },
              tintColor: "#ffffff",
              onPress: () => router.back(),
            },
          ],
          headerLeft:
            Platform.OS !== "ios"
              ? () => (
                  <Pressable onPress={() => router.back()} hitSlop={8} className="mr-2">
                    <ChevronLeft size={24} color="#FFFFFF" />
                  </Pressable>
                )
              : undefined,
          unstable_headerRightItems: () => {
            if (isEditing) {
              return [
                {
                  type: "button" as const,
                  label: "Done",
                  tintColor: "#ffffff",
                  sharesBackground: true,
                  onPress: toggleEditMode,
                },
              ]
            }
            const items = []
            if (folderId) {
              items.push({
                type: "button" as const,
                label: "New Folder",
                icon: {
                  name: "folder.badge.plus" as const,
                  type: "sfSymbol" as const,
                },
                tintColor: "#ffffff",
                sharesBackground: true,
                onPress: () => newFolderSheetRef.current?.open(null, folderId),
              })
            }
            if (noteCount > 0 || subfoldersList.length > 0) {
              items.push({
                type: "button" as const,
                label: "Edit",
                tintColor: "#ffffff",
                sharesBackground: true,
                onPress: toggleEditMode,
              })
            }
            return items
          },
          headerRight:
            Platform.OS !== "ios"
              ? () =>
                  isEditing ? (
                    <Pressable onPress={toggleEditMode} hitSlop={8}>
                      <Text className="text-[16px] font-semibold text-white">Done</Text>
                    </Pressable>
                  ) : (
                    <View className="flex-row items-center gap-2">
                      {folderId && (
                        <Pressable
                          onPress={() => newFolderSheetRef.current?.open(null, folderId)}
                          hitSlop={8}
                          className="mr-1"
                        >
                          <FolderPlus size={22} color="#FFFFFF" />
                        </Pressable>
                      )}
                      {(noteCount > 0 || subfoldersList.length > 0) && (
                        <Pressable onPress={toggleEditMode} hitSlop={8}>
                          <Text className="text-[16px] font-medium text-white">Edit</Text>
                        </Pressable>
                      )}
                    </View>
                  )
              : undefined,
        }}
      />

      {isLoading && !isReadyToLoad ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator color="#CABEFF" size="large" />
        </View>
      ) : (
        <LegendList
          data={flatItems}
          renderItem={renderItem}
          keyExtractor={keyExtractor}
          extraData={legendListExtraData}
          contentInsetAdjustmentBehavior="automatic"
          contentContainerStyle={{
            paddingHorizontal: 20,
            paddingTop: 4,
            paddingBottom: 110,
          }}
          ListHeaderComponent={
            <View className="mb-2">
              {subfoldersList.length > 0 && (
                <View className="mb-4">
                  <View className="mb-1 flex-row items-center justify-between px-1">
                    <Text className="text-[13px] font-semibold tracking-wider text-muted-foreground/60 uppercase">
                      Folders
                    </Text>
                    <Text className="text-[13px] text-muted-foreground/60">
                      {subfoldersList.length} {subfoldersList.length === 1 ? "folder" : "folders"}
                    </Text>
                  </View>
                  <DraggableFolderSection
                    folders={subfoldersList}
                    isEditing={isEditing}
                    selectedFolderIds={new Set()}
                    getFolderCount={(subId) => (subId ? (counts.byFolder[subId] ?? 0) : 0)}
                    onToggleSelect={() => {}}
                    onPressFolder={(subId) => router.push(`/folders/${subId}` as const)}
                    onLongPressFolder={(subFolder) => folderActionSheetRef.current?.open(subFolder)}
                    onDeleteFolder={(subFolder) => {
                      Alert.alert(
                        `Delete "${subFolder.name}"?`,
                        "All notes inside will be moved to the Trash.",
                        [
                          { text: "Cancel", style: "cancel" },
                          {
                            text: "Delete",
                            style: "destructive",
                            onPress: () => deleteFolderMutation.mutate(subFolder.id),
                          },
                        ],
                      )
                    }}
                    onReorder={(fromIdx, toIdx) => {
                      const next = [...subfoldersList]
                      const [moved] = next.splice(fromIdx, 1)
                      next.splice(toIdx, 0, moved)
                      reorderFoldersMutation.mutate(next.map((f) => f.id))
                    }}
                  />
                </View>
              )}

              {noteCount > 0 && (
                <View className="mb-1 flex-row items-center justify-between px-1">
                  <Text className="text-[13px] font-semibold tracking-wider text-muted-foreground/60 uppercase">
                    Notes
                  </Text>
                  <Text className="text-[13px] text-muted-foreground/60">
                    {noteCount}
                    {hasNextPage ? "+" : ""} {noteCount === 1 ? "note" : "notes"}
                  </Text>
                </View>
              )}
            </View>
          }
          onEndReached={() => {
            if (hasNextPage && !isFetchingNextPage) {
              fetchNextPage()
            }
          }}
          onEndReachedThreshold={0.5}
          ListFooterComponent={
            isFetchingNextPage ? (
              <View className="py-5">
                <ActivityIndicator color="#CABEFF" />
              </View>
            ) : null
          }
          ListEmptyComponent={
            subfoldersList.length === 0 ? (
              <View className="items-center justify-center pt-24 px-6">
                <Text className="mt-4 text-center text-lg font-semibold text-foreground">
                  {debouncedSearch
                    ? "No matching notes"
                    : isTrash
                      ? "Trash is empty"
                      : "No notes yet"}
                </Text>
                <Text className="mt-1.5 text-center text-[15px] text-muted-foreground">
                  {debouncedSearch
                    ? `Nothing matches "${debouncedSearch}"`
                    : isTrash
                      ? "Deleted notes will appear here"
                      : "Tap the button below to start writing"}
                </Text>
              </View>
            ) : null
          }
        />
      )}

      {isEditing ? (
        <NotesEditBottomBar
          selectedCount={selectedNoteIds.size}
          totalCount={notesList.length}
          isTrash={Boolean(isTrash)}
          onMove={() => moveSheetRef.current?.open()}
          onDelete={handleBatchDeleteNotes}
          onSelectAll={handleSelectAll}
          onDeselectAll={handleDeselectAll}
        />
      ) : (
        <BottomBar
          searchValue={searchValue}
          onSearchChange={setSearchValue}
          onPressNewNote={handlePressNewNote}
        />
      )}

      <NoteActionSheet
        ref={actionSheetRef}
        note={actionNote}
        isTrash={Boolean(isTrash)}
        onTogglePin={(note) => handleTogglePin(note.id)}
        onMoveToFolder={handleActionMoveToFolder}
        onTrash={(note) => handleDelete(note.id)}
        onRestore={(note) => handleRestore(note.id)}
        onDeletePermanently={(note) => handleDelete(note.id)}
      />

      <MoveNoteSheet
        ref={moveSheetRef}
        note={actionNote}
        onSelectFolder={handleSelectTargetFolder}
      />

      <NewFolderSheet
        ref={newFolderSheetRef}
        parentId={folderId ?? null}
        onCreated={(subId) => router.push(`/folders/${subId}` as const)}
      />

      <FolderActionSheet
        ref={folderActionSheetRef}
        onEdit={(subFolder) => newFolderSheetRef.current?.open(subFolder)}
        onDelete={(subFolder) => {
          Alert.alert(
            `Delete "${subFolder.name}"?`,
            "All notes inside will be moved to the Trash.",
            [
              { text: "Cancel", style: "cancel" },
              {
                text: "Delete",
                style: "destructive",
                onPress: () => deleteFolderMutation.mutate(subFolder.id),
              },
            ],
          )
        }}
        getFolderCount={(subId) => (subId ? (counts.byFolder[subId] ?? 0) : 0)}
      />
    </View>
  )
}
