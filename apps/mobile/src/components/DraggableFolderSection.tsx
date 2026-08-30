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
import { useAppTheme } from "@/hooks/useAppTheme"

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
  onLongPress?: (folder: Folder) => void
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
  onLongPress,
  onDelete,
  onReorder,
}: DraggableFolderItemProps) {
  const { colors } = useAppTheme()
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
      const finalSlot = positions.value[originalIndex]
      const targetY = (finalSlot - originalIndex) * ROW_HEIGHT

      dragTranslateY.value = withTiming(
        targetY,
        {
          duration: 200,
          easing: Easing.bezier(0.2, 0, 0, 1),
        },
        (finished) => {
          if (finished) {
            activeDragIndex.value = -1
            dragTranslateY.value = 0
            if (finalSlot !== originalIndex) {
              scheduleOnRN(() => {
                onReorder(originalIndex, finalSlot)
              })
            }
          }
        },
      )
      scheduleOnRN(triggerDropHaptic)
    })

  const rowAnimatedStyle = useAnimatedStyle(() => {
    const isDragging = activeDragIndex.value === originalIndex
    const currentSlot = positions.value[originalIndex] ?? originalIndex

    if (isDragging) {
      return {
        position: "absolute",
        top: 0,
        left: 0,
        right: 0,
        height: ROW_HEIGHT,
        transform: [
          { translateY: originalIndex * ROW_HEIGHT + dragTranslateY.value },
          { scale: 1.03 },
        ],
        backgroundColor: colors.card,
        zIndex: 999,
        shadowColor: "#000000",
        shadowOffset: { width: 0, height: 8 },
        shadowOpacity: 0.35,
        shadowRadius: 12,
        elevation: 8,
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
      {originalIndex > 0 && <View className="ml-15 h-[0.5px] bg-border" />}
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
          onLongPress={() => {
            if (!isEditing && onLongPress) {
              onLongPress(folder)
            }
          }}
          delayLongPress={260}
          className="flex-row items-center justify-between px-4 py-3.5 active:bg-accent"
          style={{ height: ROW_HEIGHT }}
        >
          <View className="flex-1 flex-row items-center">
            <Animated.View style={selectCircleStyle} className="justify-center overflow-hidden">
              {isSelected ? (
                <CircleCheck size={22} color={colors.primary} fill={colors.primary} />
              ) : (
                <Circle size={22} color={colors.mutedForeground} />
              )}
            </Animated.View>
            <View className="size-8 items-center justify-center rounded-lg">
              <FolderIcon
                name={folder.icon || DEFAULT_FOLDER_ICON}
                size={20}
                color={colors.primary}
                fill={colors.primary}
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
                  <GripVertical size={20} color={colors.mutedForeground} />
                </View>
              </GestureDetector>
            </Animated.View>
            <Animated.View style={countStyle} pointerEvents={isEditing ? "none" : "auto"}>
              <View className="flex-row items-center gap-1.5">
                <Text className="text-[15px] text-muted-foreground">{noteCount}</Text>
                <ChevronRight size={16} color={colors.mutedForeground} />
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
  onLongPressFolder?: (folder: Folder) => void
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
  onLongPressFolder,
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
      className="mt-5 overflow-hidden rounded-3xl bg-card border border-border/40"
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
          onLongPress={onLongPressFolder}
          onDelete={() => onDeleteFolder(folder)}
          onReorder={onReorder}
        />
      ))}
    </View>
  )
})
