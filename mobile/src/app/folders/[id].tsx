import { Stack, useLocalSearchParams, useRouter } from "expo-router"
import { useCallback, useMemo, useState } from "react"
import { ActivityIndicator, SectionList, Text, View } from "react-native"
import { BottomBar } from "@/components/BottomBar"
import { NoteListItem } from "@/components/NoteListItem"
import NoteSectionHeader from "@/components/NoteSectionHeader"
import type { SearchResult } from "@/db/queries"
import type { Note } from "@/db/schema"
import { useDebouncedValue } from "@/hooks/useDebouncedValue"
import { useFolder } from "@/hooks/useFolders"
import { useNotes } from "@/hooks/useNotes"
import { groupNotesByDate, type NoteSection } from "@/utils/date"

export default function FolderNotesScreen() {
  const router = useRouter()
  const { id } = useLocalSearchParams<{ id: string }>()

  const isAll = id === "all" || !id
  const isTrash = id === "trash"
  const folderId = isAll || isTrash ? undefined : id

  const { data: folder } = useFolder(folderId ?? null)

  const title = useMemo(() => {
    if (isAll) return "All Notes"
    if (isTrash) return "Trash"
    return folder?.name || "Notes"
  }, [isAll, isTrash, folder?.name])

  const [searchValue, setSearchValue] = useState("")
  const debouncedSearch = useDebouncedValue(searchValue, 150)

  const { data: notesList, isLoading } = useNotes({
    search: debouncedSearch,
    folderId: isAll ? undefined : (folderId ?? null),
    trashed: Boolean(isTrash),
  })

  const sections = useMemo(() => {
    return groupNotesByDate(notesList ?? [])
  }, [notesList])

  const noteCount = notesList?.length ?? 0

  const handlePressNote = useCallback(
    (noteId: string) => {
      router.push(`/notes/${noteId}` as const)
    },
    [router],
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

  const renderSectionHeader = ({ section: { title: sectionTitle } }: { section: NoteSection }) => (
    <NoteSectionHeader title={sectionTitle} />
  )

  const renderNoteItem = ({
    item,
    index,
    section,
  }: {
    item: Note | SearchResult
    index: number
    section: NoteSection
  }) => (
    <NoteListItem
      item={item}
      isFirst={index === 0}
      isLast={index === section.data.length - 1}
      onPress={handlePressNote}
    />
  )

  return (
    <View className="flex-1 bg-background">
      <Stack.Screen
        options={{
          title,
          headerLargeTitle: true,
          headerBackTitle: "Folders",
          headerShown: true,
        }}
      />

      {isLoading ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator size="large" color="#CABEFF" />
        </View>
      ) : (
        <SectionList
          sections={sections}
          keyExtractor={(item) => item.id}
          renderItem={renderNoteItem}
          renderSectionHeader={renderSectionHeader}
          stickySectionHeadersEnabled={false}
          contentInsetAdjustmentBehavior="automatic"
          contentContainerStyle={{
            paddingHorizontal: 20,
            paddingTop: 4,
            paddingBottom: 110,
          }}
          ListHeaderComponent={
            noteCount > 0 ? (
              <View className="mb-1 px-1">
                <Text className="text-[13px] text-muted-foreground/60">
                  {noteCount} {noteCount === 1 ? "note" : "notes"}
                </Text>
              </View>
            ) : null
          }
          ListEmptyComponent={
            <View className="items-center justify-center pt-24 px-6">
              <Text className="text-[48px]">{isTrash ? "🗑️" : "📝"}</Text>
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
