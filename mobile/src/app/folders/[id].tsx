import { Stack, useLocalSearchParams, useRouter } from "expo-router";
import { ChevronLeft } from "lucide-react-native";
import { useCallback, useMemo, useState } from "react";
import { ActivityIndicator, FlatList, Platform, Pressable, Text, View } from "react-native";
import { BottomBar } from "@/components/BottomBar";
import { NoteListItem } from "@/components/NoteListItem";
import NoteSectionHeader from "@/components/NoteSectionHeader";
import type { SearchResult } from "@/db/queries";
import type { Note } from "@/db/schema";
import { useDebouncedValue } from "@/hooks/useDebouncedValue";
import { useFolder } from "@/hooks/useFolders";
import { useNotes } from "@/hooks/useNotes";
import { groupNotesByDate } from "@/utils/date";

type FlatNoteItem =
  | { type: "header"; id: string; title: string }
  | {
      type: "note";
      id: string;
      item: Note | SearchResult;
      isFirst: boolean;
      isLast: boolean;
    };

export default function FolderNotesScreen() {
  const router = useRouter();
  const { id } = useLocalSearchParams<{ id: string }>();

  const isAll = id === "all" || !id;
  const isTrash = id === "trash";
  const folderId = isAll || isTrash ? undefined : id;

  const { data: folder } = useFolder(folderId ?? null);

  const title = useMemo(() => {
    if (isAll) return "All Notes";
    if (isTrash) return "Trash";
    return folder?.name || "Notes";
  }, [isAll, isTrash, folder?.name]);

  const [searchValue, setSearchValue] = useState("");
  const debouncedSearch = useDebouncedValue(searchValue, 150);

  const { data: notesList, isLoading } = useNotes({
    search: debouncedSearch,
    folderId: isAll ? undefined : (folderId ?? null),
    trashed: Boolean(isTrash),
  });

  const listData = useMemo<FlatNoteItem[]>(() => {
    if (!notesList || notesList.length === 0) return [];

    const sections = groupNotesByDate(notesList);
    const items: FlatNoteItem[] = [];

    for (const section of sections) {
      items.push({
        type: "header",
        id: `header-${section.title}`,
        title: section.title,
      });

      const len = section.data.length;
      section.data.forEach((note, index) => {
        items.push({
          type: "note",
          id: note.id,
          item: note,
          isFirst: index === 0,
          isLast: index === len - 1,
        });
      });
    }

    return items;
  }, [notesList]);

  const noteCount = notesList?.length ?? 0;

  const handlePressNote = useCallback(
    (noteId: string) => {
      router.push(`/notes/${noteId}` as const);
    },
    [router],
  );

  const handlePressNewNote = () => {
    if (isTrash || !folderId) {
      router.push("/notes/new" as const);
    } else {
      router.push({
        pathname: "/notes/[id]",
        params: { id: "new", folderId },
      });
    }
  };

  const renderItem = useCallback(
    ({ item }: { item: FlatNoteItem }) => {
      if (item.type === "header") {
        return <NoteSectionHeader title={item.title} />;
      }

      return (
        <NoteListItem
          item={item.item}
          isFirst={item.isFirst}
          isLast={item.isLast}
          onPress={handlePressNote}
        />
      );
    },
    [handlePressNote],
  );

  const keyExtractor = useCallback((item: FlatNoteItem) => item.id, []);

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
        }}
      />

      {isLoading ? (
        <View className="flex-1 items-center justify-center">
          <ActivityIndicator size="large" color="#CABEFF" />
        </View>
      ) : (
        <FlatList
          data={listData}
          keyExtractor={keyExtractor}
          renderItem={renderItem}
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
                  {noteCount} {noteCount === 1 ? "note" : "notes"}
                </Text>
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
  );
}
