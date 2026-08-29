import * as Haptics from "expo-haptics"
import { ChevronRight, Circle, CircleCheck, GripVertical, Trash2 } from "lucide-react-native"
import { memo, useCallback, useEffect } from "react"
import { Pressable, Text, View } from "react-native"
import { Gesture, GestureDetector } from "react-native-gesture-handler"
import Animated, {
  Easing,
  interpolate,
  type SharedValue,
  useAnimatedStyle,
  useSharedValue,
  withTiming,
} from "react-native-reanimated"
import { scheduleOnRN } from "react-native-worklets"
import { DEFAULT_FOLDER_ICON, FolderIcon } from "@/components/FolderIcon"
import { SwipeableListItem } from "@/components/SwipeableListItem"
import type { Folder } from "@/db/schema"

const ROW_HEIGHT = 58

interface DraggableFolderItemProps {
  folder: Folder
  index: number
  totalCount: number
  isFirst: boolean
  isLast: boolean
  isOnly: boolean
  isEditing: boolean
  isSelected: boolean
  noteCount: number
  activeDragIndex: SharedValue<number>
  hoverIndex: SharedValue<number>
  dragTranslateY: SharedValue<number>
  onPress: () => void
  onDelete: () => void
  onReorder: (fromIndex: number, toIndex: number) => void
}

const DraggableFolderRowItem = memo(function DraggableFolderRowItem({
  folder,
  index,
  totalCount,
  isFirst,
  isLast,
  isOnly,
  isEditing,
  isSelected,
  noteCount,
  activeDragIndex,
  hoverIndex,
  dragTranslateY,
  onPress,
  onDelete,
  onReorder,
}: DraggableFolderItemProps) {
  const editProgress = useSharedValue(isEditing ? 1 : 0)

  useEffect(() => {
    editProgress.value = withTiming(isEditing ? 1 : 0, {
      duration: 250,
      easing: Easing.bezier(0.25, 0.1, 0.25, 1),
    })
  }, [isEditing, editProgress])

  const triggerLiftHaptic = useCallback(() => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
  }, [])

  const triggerHoverHaptic = useCallback(() => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
  }, [])

  const triggerDropHaptic = useCallback(() => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
  }, [])

  const panGesture = Gesture.Pan()
    .enabled(isEditing)
    .onStart(() => {
      activeDragIndex.value = index
      hoverIndex.value = index
      dragTranslateY.value = 0
      scheduleOnRN(triggerLiftHaptic)
    })
    .onUpdate((event) => {
      dragTranslateY.value = event.translationY
      const newHover = Math.max(
        0,
        Math.min(totalCount - 1, Math.round(index + event.translationY / ROW_HEIGHT)),
      )
      if (newHover !== hoverIndex.value) {
        hoverIndex.value = newHover
        scheduleOnRN(triggerHoverHaptic)
      }
    })
    .onEnd(() => {
      const fromIdx = activeDragIndex.value
      const toIdx = hoverIndex.value

      if (fromIdx !== -1 && toIdx !== -1 && fromIdx !== toIdx) {
        const targetOffset = (toIdx - fromIdx) * ROW_HEIGHT
        dragTranslateY.value = withTiming(
          targetOffset,
          {
            duration: 120,
            easing: Easing.bezier(0.2, 0, 0, 1),
          },
          (finished) => {
            if (finished) {
              scheduleOnRN(onReorder, fromIdx, toIdx)
              activeDragIndex.value = -1
              hoverIndex.value = -1
              dragTranslateY.value = 0
            }
          },
        )
      } else {
        dragTranslateY.value = withTiming(
          0,
          {
            duration: 120,
            easing: Easing.bezier(0.2, 0, 0, 1),
          },
          (finished) => {
            if (finished) {
              activeDragIndex.value = -1
              hoverIndex.value = -1
              dragTranslateY.value = 0
            }
          },
        )
      }
      scheduleOnRN(triggerDropHaptic)
    })

  const rowAnimatedStyle = useAnimatedStyle(() => {
    const isDraggingThis = activeDragIndex.value === index
    const isDraggingAny = activeDragIndex.value !== -1

    if (isDraggingThis) {
      return {
        transform: [
          { translateY: dragTranslateY.value },
          { scale: withTiming(1.02, { duration: 120 }) },
        ],
        zIndex: 999,
        elevation: 8,
        shadowColor: "#000",
        shadowOffset: { width: 0, height: 6 },
        shadowOpacity: 0.4,
        shadowRadius: 10,
      }
    }

    if (isDraggingAny) {
      const start = activeDragIndex.value
      const hover = hoverIndex.value
      let shift = 0

      if (start < hover) {
        if (index > start && index <= hover) {
          shift = -ROW_HEIGHT
        }
      } else if (start > hover) {
        if (index >= hover && index < start) {
          shift = ROW_HEIGHT
        }
      }

      return {
        transform: [
          {
            translateY: withTiming(shift, {
              duration: 160,
              easing: Easing.bezier(0.2, 0, 0, 1),
            }),
          },
          { scale: withTiming(1, { duration: 120 }) },
        ],
        zIndex: 1,
      }
    }

    return {
      transform: [
        {
          translateY: withTiming(0, {
            duration: 160,
            easing: Easing.bezier(0.2, 0, 0, 1),
          }),
        },
        { scale: withTiming(1, { duration: 120 }) },
      ],
      zIndex: 1,
    }
  })

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
    <Animated.View
      style={rowAnimatedStyle}
      className={`overflow-hidden bg-white/7 ${roundingClass}`}
    >
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
              <GestureDetector gesture={panGesture}>
                <View className="p-2 -mr-2">
                  <GripVertical size={20} color="#8E8C99" />
                </View>
              </GestureDetector>
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
    </Animated.View>
  )
})

interface DraggableFolderSectionProps {
  folders: Folder[]
  isEditing: boolean
  selectedFolderIds: Set<string>
  getFolderCount: (folderId: string | null) => number
  onToggleSelect: (folderId: string) => void
  onPressFolder: (folderId: string) => void
  onDeleteFolder: (folder: Folder) => void
  onReorder: (fromIndex: number, toIndex: number) => void
}

export const DraggableFolderSection = memo(function DraggableFolderSection({
  folders,
  isEditing,
  selectedFolderIds,
  getFolderCount,
  onToggleSelect,
  onPressFolder,
  onDeleteFolder,
  onReorder,
}: DraggableFolderSectionProps) {
  const activeDragIndex = useSharedValue(-1)
  const hoverIndex = useSharedValue(-1)
  const dragTranslateY = useSharedValue(0)

  if (folders.length === 0) return null

  return (
    <View className="mt-5">
      {folders.map((folder, index) => {
        const isFirst = index === 0
        const isLast = index === folders.length - 1
        const isOnly = isFirst && isLast

        return (
          <DraggableFolderRowItem
            key={folder.id}
            folder={folder}
            index={index}
            totalCount={folders.length}
            isFirst={isFirst}
            isLast={isLast}
            isOnly={isOnly}
            isEditing={isEditing}
            isSelected={selectedFolderIds.has(folder.id)}
            noteCount={getFolderCount(folder.id)}
            activeDragIndex={activeDragIndex}
            hoverIndex={hoverIndex}
            dragTranslateY={dragTranslateY}
            onPress={() => {
              if (isEditing) {
                onToggleSelect(folder.id)
              } else {
                onPressFolder(folder.id)
              }
            }}
            onDelete={() => onDeleteFolder(folder)}
            onReorder={onReorder}
          />
        )
      })}
    </View>
  )
})
