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
  useDerivedValue,
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
  originalIndex: number
  totalCount: number
  isEditing: boolean
  isSelected: boolean
  noteCount: number
  positions: SharedValue<number[]>
  activeDragIndex: SharedValue<number>
  dragTranslateY: SharedValue<number>
  onPress: () => void
  onDelete: () => void
  onReorder: (fromIndex: number, toIndex: number) => void
}

const DraggableFolderRowItem = memo(function DraggableFolderRowItem({
  folder,
  originalIndex,
  totalCount,
  isEditing,
  isSelected,
  noteCount,
  positions,
  activeDragIndex,
  dragTranslateY,
  onPress,
  onDelete,
  onReorder,
}: DraggableFolderItemProps) {
  const editProgress = useDerivedValue(() => {
    return withTiming(isEditing ? 1 : 0, {
      duration: 250,
      easing: Easing.bezier(0.25, 0.1, 0.25, 1),
    })
  }, [isEditing])

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
      activeDragIndex.value = originalIndex
      dragTranslateY.value = 0
      scheduleOnRN(triggerLiftHaptic)
    })
    .onUpdate((event) => {
      dragTranslateY.value = event.translationY
      const rawSlot = originalIndex + event.translationY / ROW_HEIGHT
      const targetSlot = Math.max(0, Math.min(totalCount - 1, Math.round(rawSlot)))

      const currentSlots = [...positions.value]
      const oldSlotOfActive = currentSlots[originalIndex]

      if (targetSlot !== oldSlotOfActive) {
        const nextSlots = new Array(totalCount).fill(0)
        for (let i = 0; i < totalCount; i++) {
          if (i === originalIndex) {
            nextSlots[i] = targetSlot
          } else {
            let slot = i
            if (originalIndex < targetSlot) {
              if (i > originalIndex && i <= targetSlot) {
                slot = i - 1
              }
            } else if (originalIndex > targetSlot) {
              if (i >= targetSlot && i < originalIndex) {
                slot = i + 1
              }
            }
            nextSlots[i] = slot
          }
        }
        positions.value = nextSlots
        scheduleOnRN(triggerHoverHaptic)
      }
    })
    .onEnd(() => {
      const fromIdx = originalIndex
      const toIdx = positions.value[originalIndex]

      if (fromIdx !== toIdx) {
        scheduleOnRN(onReorder, fromIdx, toIdx)
      }
      activeDragIndex.value = -1
      dragTranslateY.value = 0
      scheduleOnRN(triggerDropHaptic)
    })

  const rowAnimatedStyle = useAnimatedStyle(() => {
    const isDraggingThis = activeDragIndex.value === originalIndex
    const currentSlot = positions.value[originalIndex] ?? originalIndex

    if (isDraggingThis) {
      return {
        position: "absolute",
        top: 0,
        left: 0,
        right: 0,
        height: ROW_HEIGHT,
        transform: [
          { translateY: originalIndex * ROW_HEIGHT + dragTranslateY.value },
          { scale: withTiming(1.02, { duration: 100 }) },
        ],
        backgroundColor: "rgba(255, 255, 255, 0.12)",
        borderRadius: 16,
        zIndex: 999,
        elevation: 8,
        shadowColor: "#000",
        shadowOffset: { width: 0, height: 6 },
        shadowOpacity: 0.4,
        shadowRadius: 10,
      }
    }

    return {
      position: "absolute",
      top: 0,
      left: 0,
      right: 0,
      height: ROW_HEIGHT,
      transform: [
        {
          translateY: withTiming(currentSlot * ROW_HEIGHT, {
            duration: 160,
            easing: Easing.bezier(0.2, 0, 0, 1),
          }),
        },
        { scale: 1 },
      ],
      backgroundColor: "transparent",
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

  return (
    <Animated.View style={rowAnimatedStyle} className="overflow-hidden">
      <View className="ml-15 h-[0.5px] bg-white/8" />
      <SwipeableListItem
        enabled={!isEditing}
        rightAction={{
          label: "Delete",
          color: "#D94C5C",
          icon: <Trash2 size={20} color="#FFFFFF" />,
          onPress: onDelete,
        }}
      >
        <Pressable
          onPress={onPress}
          className="flex-row items-center justify-between px-4 py-3.5 active:bg-white/12"
          style={{ height: ROW_HEIGHT }}
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
  const dragTranslateY = useSharedValue(0)
  const positions = useSharedValue<number[]>(folders.map((_, i) => i))

  useEffect(() => {
    positions.value = folders.map((_, i) => i)
  }, [folders, positions])

  if (folders.length === 0) return null

  return (
    <View
      className="mt-5 overflow-hidden rounded-3xl bg-white/7"
      style={{ height: folders.length * ROW_HEIGHT }}
    >
      {folders.map((folder, index) => (
        <DraggableFolderRowItem
          key={folder.id}
          folder={folder}
          originalIndex={index}
          totalCount={folders.length}
          isEditing={isEditing}
          isSelected={selectedFolderIds.has(folder.id)}
          noteCount={getFolderCount(folder.id)}
          positions={positions}
          activeDragIndex={activeDragIndex}
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
      ))}
    </View>
  )
})
