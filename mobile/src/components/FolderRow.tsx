import { ChevronRight, Circle, CircleCheck, GripVertical, Trash2 } from "lucide-react-native"
import { memo, useEffect } from "react"
import { Pressable, Text, View } from "react-native"
import Animated, {
  Easing,
  interpolate,
  useAnimatedStyle,
  useSharedValue,
  withTiming,
} from "react-native-reanimated"
import { DEFAULT_FOLDER_ICON, FolderIcon } from "@/components/FolderIcon"
import { SwipeableListItem } from "@/components/SwipeableListItem"
import type { Folder } from "@/db/schema"

export interface FolderRowProps {
  folder: Folder
  isFirst: boolean
  isLast: boolean
  isOnly: boolean
  isEditing: boolean
  isSelected: boolean
  noteCount: number
  onPress: () => void
  onDelete: () => void
}

export const FolderRow = memo(function FolderRow({
  folder,
  isFirst,
  isLast,
  isOnly,
  isEditing,
  isSelected,
  noteCount,
  onPress,
  onDelete,
}: FolderRowProps) {
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

  const handleStyle = useAnimatedStyle(() => ({
    opacity: editProgress.value,
    transform: [{ scale: interpolate(editProgress.value, [0, 1], [0.6, 1]) }],
  }))

  const countStyle = useAnimatedStyle(() => ({
    opacity: interpolate(editProgress.value, [0, 1], [1, 0]),
    transform: [{ scale: interpolate(editProgress.value, [0, 1], [1, 0.8]) }],
  }))

  const roundingClass = isOnly
    ? "rounded-3xl"
    : isFirst
      ? "rounded-t-3xl"
      : isLast
        ? "rounded-b-3xl"
        : ""

  return (
    <View className={`overflow-hidden bg-white/7 ${roundingClass} ${isFirst ? "mt-5" : ""}`}>
      {!isFirst && <View className="ml-15 h-[0.5px] bg-white/8" />}
      <SwipeableListItem
        enabled={!isEditing}
        rounded={isOnly ? "only" : isFirst ? "first" : isLast ? "last" : "middle"}
        rightAction={{
          label: "Delete",
          color: "#D94C5C",
          icon: <Trash2 size={20} color="#FFFFFF" />,
          onPress: onDelete,
        }}
      >
        <Pressable
          onPress={onPress}
          className={`flex-row items-center justify-between px-4 py-3.5 active:bg-white/12 ${roundingClass}`}
        >
          <View className="flex-1 flex-row items-center">
            <Animated.View style={selectCircleStyle} className="justify-center overflow-hidden">
              {isSelected ? (
                <CircleCheck size={22} color="#CABEFF" fill="#CABEFF" />
              ) : (
                <Circle size={22} color="#8E8C99" />
              )}
            </Animated.View>
            <View className="size-8 items-center justify-center rounded-lg">
              <FolderIcon
                name={folder.icon || DEFAULT_FOLDER_ICON}
                size={20}
                color="#CABEFF"
                fill="#CABEFF"
              />
            </View>
            <Text className="ml-3 flex-1 text-[17px] text-foreground" numberOfLines={1}>
              {folder.name}
            </Text>
          </View>

          <View className="items-end justify-center">
            <Animated.View
              style={[handleStyle, { position: "absolute", right: 0 }]}
              pointerEvents={isEditing ? "auto" : "none"}
            >
              <GripVertical size={20} color="#8E8C99" />
            </Animated.View>
            <Animated.View style={countStyle} pointerEvents={isEditing ? "none" : "auto"}>
              <View className="flex-row items-center gap-1.5">
                <Text className="text-[15px] text-muted-foreground">{noteCount}</Text>
                <ChevronRight size={16} color="#8E8C99" />
              </View>
            </Animated.View>
          </View>
        </Pressable>
      </SwipeableListItem>
    </View>
  )
})
