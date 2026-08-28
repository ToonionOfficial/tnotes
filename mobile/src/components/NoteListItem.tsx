import { Pin } from "lucide-react-native"
import { memo } from "react"
import { Pressable, Text, View } from "react-native"
import type { SearchResult } from "@/db/queries"
import type { Note } from "@/db/schema"
import { formatNoteTime } from "@/utils/date"
import { stripHtml } from "@/utils/text"

interface NoteListItemProps {
  item: Note | SearchResult
  isFirst: boolean
  isLast: boolean
  onPress: (noteId: string) => void
}

export const NoteListItem = memo(function NoteListItem({
  item,
  isFirst,
  isLast,
  onPress,
}: NoteListItemProps) {
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
        onPress={() => onPress(item.id)}
        className={`${roundingClass} bg-white/[0.07] px-4 py-3 active:bg-white/12`}
      >
        <View className="flex-row items-center gap-2">
          {item.pinned && <Pin size={12} color="#CABEFF" fill="#CABEFF" />}
          <Text numberOfLines={1} className="flex-1 text-[16px] font-semibold text-foreground">
            {item.title || "Untitled Note"}
          </Text>
        </View>
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
      {!isLast && <View className="ml-4 h-[0.5px] bg-white/8" />}
    </View>
  )
})
