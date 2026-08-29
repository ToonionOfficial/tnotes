import { ArchiveRestore, Circle, CircleCheck, Pin, Trash2 } from "lucide-react-native"
import { memo, useEffect } from "react"
import { Pressable, Text, View } from "react-native"
import Animated, {
  Easing,
  interpolate,
  useAnimatedStyle,
  useSharedValue,
  withTiming,
} from "react-native-reanimated"
import { SwipeableListItem } from "@/components/SwipeableListItem"
import type { SearchResult } from "@/db/queries"
import type { Note } from "@/db/schema"
import { formatNoteTime } from "@/utils/date"
import { stripHtml } from "@/utils/text"

interface NoteListItemProps {
  item: Note | SearchResult
  isFirst: boolean
  isLast: boolean
  isTrash?: boolean
  isEditing?: boolean
  isSelected?: boolean
  onPress: (noteId: string) => void
  onTogglePin?: (noteId: string) => void
  onRestore?: (noteId: string) => void
  onDelete?: (noteId: string) => void
}

export const NoteListItem = memo(function NoteListItem({
  item,
  isFirst,
  isLast,
  isTrash = false,
  isEditing = false,
  isSelected = false,
  onPress,
  onTogglePin,
  onRestore,
  onDelete,
}: NoteListItemProps) {
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

  const isOnly = isFirst && isLast
  let roundingClass = "rounded-none"
  if (isOnly) {
    roundingClass = "rounded-3xl"
  } else if (isFirst) {
    roundingClass = "rounded-t-3xl"
  } else if (isLast) {
    roundingClass = "rounded-b-3xl"
  }

  const previewText =
    "snippet" in item && item.snippet ? stripHtml(item.snippet) : stripHtml(item.body)

  const noteRow = (
    <Pressable
      onPress={() => onPress(item.id)}
      className={`${roundingClass} flex-row items-center bg-white/[0.07] px-4 py-3 active:bg-white/12`}
    >
      <Animated.View style={selectCircleStyle} className="justify-center overflow-hidden">
        {isSelected ? (
          <CircleCheck size={22} color="#CABEFF" fill="#CABEFF" />
        ) : (
          <Circle size={22} color="#8E8C99" />
        )}
      </Animated.View>

      <View className="flex-1">
        <View className="flex-row items-center gap-2">
          {!isTrash && item.pinned && <Pin size={12} color="#CABEFF" fill="#CABEFF" />}
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
      </View>
    </Pressable>
  )

  const leftAction = isTrash
    ? onRestore
      ? {
          label: "Restore",
          color: "#3F8CFF",
          icon: <ArchiveRestore size={20} color="#FFFFFF" />,
          onPress: () => onRestore(item.id),
        }
      : undefined
    : onTogglePin
      ? {
          label: item.pinned ? "Unpin" : "Pin",
          color: "#6C5DD3",
          icon: <Pin size={20} color="#FFFFFF" fill={item.pinned ? "#FFFFFF" : "none"} />,
          onPress: () => onTogglePin(item.id),
        }
      : undefined

  return (
    <SwipeableListItem
      enabled={!isEditing}
      leftAction={leftAction}
      rightAction={
        onDelete
          ? {
              label: isTrash ? "Delete" : "Trash",
              color: "#D94C5C",
              icon: <Trash2 size={20} color="#FFFFFF" />,
              onPress: () => onDelete(item.id),
            }
          : undefined
      }
      rounded={isOnly ? "only" : isFirst ? "first" : isLast ? "last" : "middle"}
    >
      <View>
        {noteRow}
        {!isLast && <View className="ml-4 h-[0.5px] bg-white/8" />}
      </View>
    </SwipeableListItem>
  )
})
