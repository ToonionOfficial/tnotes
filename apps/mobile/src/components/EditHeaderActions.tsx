import { Button, Host, Icon } from "@expo/ui"
import {
  buttonBorderShape,
  buttonStyle,
  controlSize,
  disabled as disabledModifier,
  foregroundStyle,
  tint,
} from "@expo/ui/swift-ui/modifiers"
import { Trash2 } from "lucide-react-native"
import { memo } from "react"
import { Platform, Pressable, Text, View } from "react-native"

interface EditHeaderActionsProps {
  isSelected: boolean
  onDelete: () => void
  onDone: () => void
  deleteColor?: string
  deleteDisabledColor?: string
  doneBgColor?: string
  doneTextColor?: string
}

const TRASH_ICON = Icon.select({
  ios: "trash",
  android: import("@expo/material-symbols/delete.xml"),
})

export const EditHeaderActions = memo(function EditHeaderActions({
  isSelected,
  onDelete,
  onDone,
  deleteColor,
  deleteDisabledColor,
  doneBgColor,
  doneTextColor,
}: EditHeaderActionsProps) {
  const currentDeleteIcon = isSelected
    ? (deleteColor ?? "#FF453A")
    : (deleteDisabledColor ?? "rgba(255, 255, 255, 0.3)")

  const currentDoneBg = doneBgColor ?? "rgba(255, 255, 255, 0.12)"
  const currentDoneText = doneTextColor ?? "#FFFFFF"

  if (Platform.OS === "ios") {
    return (
      <View className="flex-row items-center gap-2.5">
        {/* Delete Button */}
        <Host matchContents ignoreSafeArea="all">
          <Button
            variant="text"
            onPress={() => {
              if (isSelected) {
                onDelete()
              }
            }}
            modifiers={[
              buttonStyle("glass"),
              buttonBorderShape("circle"),
              controlSize("large"),
              disabledModifier(!isSelected),
              foregroundStyle(currentDeleteIcon),
              tint(currentDeleteIcon),
            ]}
          >
            <Icon name={TRASH_ICON} color={currentDeleteIcon} size={18} />
          </Button>
        </Host>

        {/* Done Button */}
        <Host matchContents ignoreSafeArea="all">
          <Button
            variant="text"
            label="Done"
            onPress={onDone}
            modifiers={[
              buttonStyle("glass"),
              buttonBorderShape("capsule"),
              controlSize("large"),
              tint(currentDoneText),
              foregroundStyle(currentDoneText),
            ]}
          />
        </Host>
      </View>
    )
  }

  // Android Fallback
  return (
    <View className="flex-row items-center gap-2.5">
      <Pressable
        onPress={onDelete}
        disabled={!isSelected}
        hitSlop={8}
        className={`size-11 items-center justify-center rounded-full active:opacity-60 ${
          isSelected ? "bg-[#FF453A]/15" : "bg-white/5 opacity-40"
        }`}
      >
        <Trash2 size={20} color={currentDeleteIcon} />
      </Pressable>

      <Pressable
        onPress={onDone}
        hitSlop={8}
        className="h-11 items-center justify-center rounded-full px-4 active:opacity-60"
        style={{ backgroundColor: currentDoneBg }}
      >
        <Text style={{ color: currentDoneText }} className="text-[15px] font-semibold">
          Done
        </Text>
      </Pressable>
    </View>
  )
})
