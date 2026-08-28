import { Stack, useLocalSearchParams, useRouter } from "expo-router"
import { Pin } from "lucide-react-native"
import { useEffect, useMemo, useState } from "react"
import { ActivityIndicator, Pressable, SectionList, Text, View } from "react-native"
import { BottomBar } from "@/components/BottomBar"
import type { SearchResult } from "@/db/queries"
import type { Note } from "@/db/schema"
import { useFolder } from "@/hooks/useFolders"
import { useNotes } from "@/hooks/useNotes"
import { formatNoteTime, groupNotesByDate, type NoteSection } from "@/utils/date"
import { stripHtml } from "@/utils/text"

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
  const [debouncedSearch, setDebouncedSearch] = useState("")

  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedSearch(searchValue)
    }, 150)
    return () => clearTimeout(timer)
  }, [searchValue])

  const { data: notesList, isLoading } = useNotes({
    search: debouncedSearch,
    folderId: isAll ? undefined : (folderId ?? null),
    trashed: Boolean(isTrash),
  })

  const sections = useMemo(() => {
    return groupNotesByDate(notesList ?? [])
  }, [notesList])

  const noteCount = notesList?.length ?? 0

  const handlePressNote = (noteId: string) => {
    router.push(`/notes/${noteId}` as const)
  }

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
    <View className="mb-1.5 mt-5 px-1">
      <Text className="text-[13px] font-semibold uppercase tracking-wider text-muted-foreground/80">
        {sectionTitle}
      </Text>
    </View>
  )

  const renderNoteItem = ({
    item,
    index,
    section,
  }: {
    item: Note | SearchResult
    index: number
    section: NoteSection
  }) => {
    const isFirst = index === 0
    const isLast = index === section.data.length - 1
    const isOnly = isFirst && isLast

    let roundingClass = "rounded-none"
    if (isOnly) {
      roundingClass = "rounded-2xl"
    } else if (isFirst) {
      roundingClass = "rounded-t-2xl"
    } else if (isLast) {
      roundingClass = "rounded-b-2xl"
    }

    const previewText =
      "snippet" in item && item.snippet ? stripHtml(item.snippet) : stripHtml(item.body)

    return (
      <View>
        <Pressable
          onPress={() => handlePressNote(item.id)}
          className={`${roundingClass} bg-white/[0.07] px-4 py-3 active:bg-white/[0.12]`}
        >
          {/* Title Row */}
          <View className="flex-row items-center gap-2">
            {item.pinned && <Pin size={12} color="#CABEFF" fill="#CABEFF" />}
            <Text numberOfLines={1} className="flex-1 text-[16px] font-semibold text-foreground">
              {item.title || "Untitled Note"}
            </Text>
          </View>

          {/* Subtitle: time + preview */}
          <View className="mt-0.5 flex-row items-center gap-1.5">
            <Text className="text-[13px] text-muted-foreground/60">
              {formatNoteTime(item.updatedAt)}
            </Text>
            {previewText.length > 0 && (
              <>
                <Text className="text-[13px] text-muted-foreground/40">·</Text>
                <Text numberOfLines={1} className="flex-1 text-[13px] text-muted-foreground/80">
                  {previewText}
                </Text>
              </>
            )}
          </View>
        </Pressable>

        {!isLast && <View className="ml-4 h-[0.5px] bg-white/[0.08]" />}
      </View>
    )
  }

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
