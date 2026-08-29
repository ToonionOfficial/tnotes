import * as Haptics from "expo-haptics"
import { ArchiveRestore, FolderInput, Trash2 } from "lucide-react-native"
import { memo } from "react"
import { Pressable, Text, View } from "react-native"
import { useSafeAreaInsets } from "react-native-safe-area-context"

export interface NotesEditBottomBarProps {
  selectedCount: number
  totalCount: number
  isTrash?: boolean
  onMove: () => void
  onDelete: () => void
  onSelectAll: () => void
  onDeselectAll: () => void
}

export const NotesEditBottomBar = memo(function NotesEditBottomBar({
  selectedCount,
  totalCount,
  isTrash = false,
  onMove,
  onDelete,
  onSelectAll,
  onDeselectAll,
}: NotesEditBottomBarProps) {
  const insets = useSafeAreaInsets()
  const isSelected = selectedCount > 0
  const isAllSelected = selectedCount === totalCount && totalCount > 0

  const handleToggleSelectAll = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    if (isAllSelected) {
      onDeselectAll()
    } else {
      onSelectAll()
    }
  }

  return (
    <View
      style={{
        paddingBottom: Math.max(insets.bottom, 16),
      }}
      className="absolute bottom-0 left-0 right-0 border-t border-white/10 bg-[#1C1B20]/95 px-6 pt-3 backdrop-blur-xl"
    >
      <View className="flex-row items-center justify-between">
        {/* Left: Move / Restore Action */}
        <Pressable
          onPress={() => {
            if (isSelected) {
              void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
              onMove()
            }
          }}
          disabled={!isSelected}
          hitSlop={8}
          className={`flex-row items-center gap-2 py-2 ${
            isSelected ? "active:opacity-60" : "opacity-35"
          }`}
        >
          {isTrash ? (
            <ArchiveRestore size={20} color={isSelected ? "#3F8CFF" : "#8E8C99"} />
          ) : (
            <FolderInput size={20} color={isSelected ? "#CABEFF" : "#8E8C99"} />
          )}
          <Text
            className={`text-[15px] font-medium ${
              isSelected ? (isTrash ? "text-[#3F8CFF]" : "text-[#CABEFF]") : "text-muted-foreground"
            }`}
          >
            {isTrash ? "Restore" : "Move"}
            {selectedCount > 0 ? ` (${selectedCount})` : ""}
          </Text>
        </Pressable>

        {/* Center: Select All / Deselect All */}
        <Pressable
          onPress={handleToggleSelectAll}
          hitSlop={8}
          className="rounded-full bg-white/[0.08] px-3.5 py-1.5 active:opacity-60"
        >
          <Text className="text-[13px] font-medium text-white/80">
            {isAllSelected ? "Deselect All" : "Select All"}
          </Text>
        </Pressable>

        {/* Right: Delete / Trash Action */}
        <Pressable
          onPress={() => {
            if (isSelected) {
              void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
              onDelete()
            }
          }}
          disabled={!isSelected}
          hitSlop={8}
          className={`flex-row items-center gap-2 py-2 ${
            isSelected ? "active:opacity-60" : "opacity-35"
          }`}
        >
          <Text
            className={`text-[15px] font-medium ${
              isSelected ? "text-[#FF6B6B]" : "text-muted-foreground"
            }`}
          >
            {isTrash ? "Delete" : "Trash"}
            {selectedCount > 0 ? ` (${selectedCount})` : ""}
          </Text>
          <Trash2 size={20} color={isSelected ? "#FF6B6B" : "#8E8C99"} />
        </Pressable>
      </View>
    </View>
  )
})
