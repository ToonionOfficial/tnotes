import { LegendList } from "@legendapp/list/react-native"
import * as Haptics from "expo-haptics"
import { Stack, useLocalSearchParams, useRouter } from "expo-router"
import { ChevronLeft, Trash2 } from "lucide-react-native"
import { useCallback, useEffect, useMemo, useState } from "react"
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
import { NoteListItem } from "@/components/NoteListItem"
import NoteSectionHeader from "@/components/NoteSectionHeader"
import type { SearchResult } from "@/db/queries"
import type { Note } from "@/db/schema"
import { useDebouncedValue } from "@/hooks/useDebouncedValue"
import { useFolder } from "@/hooks/useFolders"
import {
  useBatchDeleteNotesPermanently,
  useBatchTrashNotes,
  useDeleteNotePermanently,
  useNotes,
  useRestoreNote,
  useTogglePinNote,
  useTrashNote,
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
  const togglePinNote = useTogglePinNote()
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
      Alert.alert(`Move ${count} ${count === 1 ? "Note" : "Notes"} to Trash?`, "", [
        { text: "Cancel", style: "cancel" },
        {
          text: "Move to Trash",
          style: "destructive",
          onPress: () => {
            batchTrashNotes.mutate(ids)
            setSelectedNoteIds(new Set())
          },
        },
      ])
    }
  }, [batchDeleteNotesPermanently, batchTrashNotes, isTrash, selectedNoteIds])

  const handleTogglePin = useCallback(
    (noteId: string) => {
      togglePinNote.mutate(noteId)
    },
    [togglePinNote],
  )

  const handleRestore = useCallback(
    (noteId: string) => {
      restoreNote.mutate(noteId)
    },
    [restoreNote],
  )

  const handleDelete = useCallback(
    (noteId: string) => {
      if (isTrash) {
        deleteNotePermanently.mutate(noteId)
      } else {
        trashNote.mutate(noteId)
      }
    },
    [isTrash, deleteNotePermanently, trashNote],
  )

  const listData = useMemo<FlatNoteItem[]>(() => {
    if (notesList.length === 0) return []

    const sections = groupNotesByDate(notesList, { ignorePinned: Boolean(isTrash) })
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
      handleNotePress,
      isEditing,
      isTrash,
      selectedNoteIds,
    ],
  )

  const keyExtractor = useCallback((item: FlatNoteItem) => item.id, [])

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
          unstable_headerRightItems: () =>
            isEditing
              ? [
                  {
                    type: "button" as const,
                    label: "Delete",
                    icon: {
                      name: "trash",
                      type: "sfSymbol" as const,
                    },
                    tintColor: "#FF3B30",
                    sharesBackground: true,
                    onPress: handleBatchDeleteNotes,
                  },
                  {
                    type: "button" as const,
                    label: "Done",
                    tintColor: "#ffffff",
                    sharesBackground: true,
                    onPress: toggleEditMode,
                  },
                ]
              : noteCount > 0
                ? [
                    {
                      type: "button" as const,
                      label: "Edit",
                      tintColor: "#ffffff",
                      sharesBackground: true,
                      onPress: toggleEditMode,
                    },
                  ]
                : [],
          headerRight:
            Platform.OS !== "ios"
              ? () =>
                  isEditing ? (
                    <View className="flex-row items-center">
                      <Pressable
                        onPress={handleBatchDeleteNotes}
                        disabled={selectedNoteIds.size === 0}
                        hitSlop={8}
                        className="px-2 py-1 active:opacity-60"
                      >
                        <Trash2 size={20} color="#FF3B30" />
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
                  ) : noteCount > 0 ? (
                    <Pressable
                      onPress={toggleEditMode}
                      hitSlop={8}
                      className="px-2 py-1 active:opacity-60"
                    >
                      <Text className="text-[17px] font-medium text-white">Edit</Text>
                    </Pressable>
                  ) : undefined
              : undefined,
        }}
      />

      {!isReadyToLoad || isLoading ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator size="large" color="#CABEFF" />
        </View>
      ) : (
        <LegendList
          data={listData}
          keyExtractor={keyExtractor}
          renderItem={renderItem}
          estimatedItemSize={68}
          getFixedItemSize={(item) => (item.type === "header" ? 36 : undefined)}
          recycleItems={true}
          extraData={{ isEditing, selectedNoteIds }}
          className="flex-1"
          contentInsetAdjustmentBehavior="automatic"
          keyboardShouldPersistTaps="handled"
          contentContainerStyle={{
            paddingHorizontal: 20,
            paddingTop: 4,
            paddingBottom: 110,
          }}
          ListHeaderComponent={
            noteCount > 0 ? (
              <View className="mb-1 px-1">
                <Text className="text-[13px] text-muted-foreground/60">
                  {noteCount}
                  {hasNextPage ? "+" : ""} {noteCount === 1 ? "note" : "notes"}
                </Text>
              </View>
            ) : null
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
          }
        />
      )}

      <BottomBar
        searchValue={searchValue}
        onSearchChange={setSearchValue}
        onPressNewNote={handlePressNewNote}
      />
    </View>
  )
}
