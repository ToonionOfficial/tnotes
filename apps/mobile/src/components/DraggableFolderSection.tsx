import * as Haptics from "expo-haptics"
import { ChevronRight, Circle, CircleCheck, GripVertical, Trash2 } from "lucide-react-native"
import { memo, useCallback, useEffect } from "react"
import { Pressable, Text, View } from "react-native"
import { Gesture, GestureDetector } from "react-native-gesture-handler"
import Animated, {
  Easing,
  interpolate,
  runOnJS,
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

/**
 * O(1) slot math for drag-and-drop: given the dragged row (`active`) and its
 * current target slot (`target`), returns the slot a row at `index` should
 * occupy. Rows between the old and new position shift by exactly one; every
 * other row stays put. Runs on the UI thread once per row per frame with no
 * allocations — the previous implementation copied/looped an N-length array
 * per frame and invalidated all N row worklets on every write, which is what
 * killed the app with 100s of folders.
 */
function slotForIndex(index: number, active: number, target: number): number {
  "worklet"
  if (active === -1 || target === -1 || index === active) return index
  if (active < target) {
    return index > active && index <= target ? index - 1 : index
  }
  return index >= target && index < active ? index + 1 : index
}

interface DraggableFolderItemProps {
  folder: Folder
  originalIndex: number
  totalCount: number
  isEditing: boolean
  isSelected: boolean
  noteCount: number
  activeIndex: SharedValue<number>
  dragTranslateY: SharedValue<number>
  targetSlot: SharedValue<number>
  onToggleSelect: (folderId: string) => void
  onPressFolder: (folderId: string) => void
  onLongPressFolder?: (folder: Folder) => void
  onDeleteFolder: (folder: Folder) => void
  onReorder: (fromIndex: number, toIndex: number) => void
}

const DraggableFolderRowItem = memo(function DraggableFolderRowItem({
  folder,
  originalIndex,
  totalCount,
  isEditing,
  isSelected,
  noteCount,
  activeIndex,
  dragTranslateY,
  targetSlot,
  onToggleSelect,
  onPressFolder,
  onLongPressFolder,
  onDeleteFolder,
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
      activeIndex.value = originalIndex
      dragTranslateY.value = 0
      targetSlot.value = originalIndex
      scheduleOnRN(triggerLiftHaptic)
    })
    .onUpdate((event) => {
      dragTranslateY.value = event.translationY
      const rawSlot = originalIndex + event.translationY / ROW_HEIGHT
      const clamped = Math.max(0, Math.min(totalCount - 1, Math.round(rawSlot)))
      if (clamped !== targetSlot.value) {
        targetSlot.value = clamped
        scheduleOnRN(triggerHoverHaptic)
      }
    })
    .onEnd(() => {
      const from = originalIndex
      const to = targetSlot.value
      if (to !== -1 && to !== from) {
        // Parent re-render (new order) releases the drag pose via the
        // section's reset effect — no withTiming settle fighting it.
        // runOnJS (not scheduleOnRN): onReorder is a plain JS callback and
        // worklets rejects locally-defined closures passed to scheduleOnRN.
        runOnJS(onReorder)(from, to)
      } else {
        activeIndex.value = -1
        dragTranslateY.value = 0
        targetSlot.value = -1
      }
      scheduleOnRN(triggerDropHaptic)
    })

  const rowAnimatedStyle = useAnimatedStyle(() => {
    const active = activeIndex.value
    const dragY = dragTranslateY.value
    const target = targetSlot.value
    const isDragging = active === originalIndex

    if (isDragging) {
      return {
        position: "absolute",
        top: 0,
        left: 0,
        right: 0,
        height: ROW_HEIGHT,
        transform: [{ translateY: originalIndex * ROW_HEIGHT + dragY }, { scale: 1.03 }],
        backgroundColor: colors.card,
        zIndex: 999,
        shadowColor: "#000000",
        shadowOffset: { width: 0, height: 8 },
        shadowOpacity: 0.35,
        shadowRadius: 12,
        elevation: 8,
      }
    }

    // Direct mapping, no withTiming: withTiming inside useAnimatedStyle
    // re-creates an animation object on every frame for every row, which
    // thrashed the UI thread with 100s of rows mounted.
    const slot = slotForIndex(originalIndex, active, target)

    return {
      position: "absolute",
      top: 0,
      left: 0,
      right: 0,
      height: ROW_HEIGHT,
      transform: [{ translateY: slot * ROW_HEIGHT }, { scale: 1 }],
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

  const handlePress = () => {
    if (isEditing) {
      onToggleSelect(folder.id)
    } else {
      onPressFolder(folder.id)
    }
  }

  const handleLongPress = () => {
    if (!isEditing && onLongPressFolder) {
      onLongPressFolder(folder)
    }
  }

  const handleDelete = () => {
    onDeleteFolder(folder)
  }

  return (
    <Animated.View style={rowAnimatedStyle} className="overflow-hidden">
      {originalIndex > 0 && <View className="ml-15 h-[0.5px] bg-border" />}
      <SwipeableListItem
        enabled={!isEditing}
        rightAction={{
          label: "Delete",
          color: "#D94C5C",
          icon: <Trash2 size={20} color="#FFFFFF" />,
          onPress: handleDelete,
        }}
      >
        <Pressable
          onPress={handlePress}
          onLongPress={handleLongPress}
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
  const activeIndex = useSharedValue(-1)
  const dragTranslateY = useSharedValue(0)
  const targetSlot = useSharedValue(-1)

  // A new order from the parent (drop committed, refetch, or background
  // change) releases any in-flight drag pose so rows reconcile to props.
  // biome-ignore lint/correctness/useExhaustiveDependencies: folders is an unread change signal; shared values are stable refs.
  useEffect(() => {
    activeIndex.value = -1
    dragTranslateY.value = 0
    targetSlot.value = -1
  }, [folders, activeIndex, dragTranslateY, targetSlot])

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
          activeIndex={activeIndex}
          dragTranslateY={dragTranslateY}
          targetSlot={targetSlot}
          onToggleSelect={onToggleSelect}
          onPressFolder={onPressFolder}
          onLongPressFolder={onLongPressFolder}
          onDeleteFolder={onDeleteFolder}
          onReorder={onReorder}
        />
      ))}
    </View>
  )
})
