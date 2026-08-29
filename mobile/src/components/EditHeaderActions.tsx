import { Button, Host, Icon } from "@expo/ui"
import {
  background,
  buttonBorderShape,
  buttonStyle,
  controlSize,
  disabled as disabledModifier,
  foregroundStyle,
  tint,
} from "@expo/ui/swift-ui/modifiers"
import { Trash2 } from "lucide-react-native"
import { Platform, Pressable, Text, View } from "react-native"

export interface EditHeaderActionsProps {
  selectedCount: number
  onDelete: () => void
  onDone: () => void
  deleteBgColor?: string
  deleteMutedBgColor?: string
  deleteIconColor?: string
  deleteMutedIconColor?: string
  doneBgColor?: string
  doneTextColor?: string
}

const TRASH_ICON = Icon.select({
  ios: "trash",
  android: import("@expo/material-symbols/delete.xml"),
})

export function EditHeaderActions({
  selectedCount,
  onDelete,
  onDone,
  deleteBgColor,
  deleteMutedBgColor,
  deleteIconColor,
  deleteMutedIconColor,
  doneBgColor,
  doneTextColor,
}: EditHeaderActionsProps) {
  const isSelected = selectedCount > 0

  const activeDeleteBg = deleteBgColor ?? "rgba(255, 59, 48, 0.22)"
  const mutedDeleteBg = deleteMutedBgColor ?? "rgba(255, 59, 48, 0.08)"
  const currentDeleteBg = isSelected ? activeDeleteBg : mutedDeleteBg

  const activeDeleteIcon = deleteIconColor ?? "#FF3B30"
  const mutedDeleteIcon = deleteMutedIconColor ?? "rgba(255, 59, 48, 0.4)"
  const currentDeleteIcon = isSelected ? activeDeleteIcon : mutedDeleteIcon

  const currentDoneBg = doneBgColor ?? "rgba(255, 255, 255, 0.12)"
  const currentDoneText = doneTextColor ?? "#FFFFFF"

  if (Platform.OS === "ios") {
    return (
      <Host matchContents ignoreSafeArea="all">
        <View className="flex-row items-center gap-2">
          {/* Delete Button */}
          <Button
            variant="filled"
            onPress={() => {
              if (isSelected) {
                onDelete()
              }
            }}
            modifiers={[
              buttonStyle("glass"),
              buttonBorderShape("circle"),
              controlSize("regular"),
              disabledModifier(!isSelected),
              background(currentDeleteBg),
              foregroundStyle(currentDeleteIcon),
              tint(currentDeleteIcon),
            ]}
          >
            <Icon name={TRASH_ICON} color={currentDeleteIcon} size={16} />
          </Button>

          {/* Done Button */}
          <Button
            variant="filled"
            label="Done"
            onPress={onDone}
            modifiers={[
              buttonStyle("glass"),
              buttonBorderShape("capsule"),
              controlSize("regular"),
              background(currentDoneBg),
              foregroundStyle(currentDoneText),
            ]}
          />
        </View>
      </Host>
    )
  }

  // Android Fallback
  return (
    <View className="flex-row items-center gap-2">
      <Pressable
        onPress={() => {
          if (isSelected) onDelete()
        }}
        disabled={!isSelected}
        hitSlop={8}
        style={{
          backgroundColor: currentDeleteBg,
        }}
        className="size-9 items-center justify-center rounded-full active:opacity-60"
      >
        <Trash2 size={18} color={currentDeleteIcon} />
      </Pressable>
      <Pressable
        onPress={onDone}
        hitSlop={8}
        style={{
          backgroundColor: currentDoneBg,
        }}
        className="h-9 items-center justify-center rounded-full px-4 active:opacity-60"
      >
        <Text style={{ color: currentDoneText }} className="text-[15px] font-semibold">
          Done
        </Text>
      </Pressable>
    </View>
  )
}
