import { Button, Host, Icon } from "@expo/ui"
import {
  buttonBorderShape,
  buttonStyle,
  controlSize,
  foregroundStyle,
} from "@expo/ui/swift-ui/modifiers"
import * as Haptics from "expo-haptics"
import { X } from "lucide-react-native"
import { memo } from "react"
import { Platform, Pressable, Text, View } from "react-native"
import { useSafeAreaInsets } from "react-native-safe-area-context"
import { useAppTheme } from "@/hooks/useAppTheme"

interface SettingsHeaderProps {
  onClose: () => void
}

const CLOSE_ICON = Icon.select({
  ios: "xmark",
  android: import("@expo/material-symbols/close.xml"),
})

export const SettingsHeader = memo(function SettingsHeader({ onClose }: SettingsHeaderProps) {
  const insets = useSafeAreaInsets()
  const { colors } = useAppTheme()

  const handleClose = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    onClose()
  }

  return (
    <View style={{ paddingTop: insets.top + 8 }} className="px-5">
      <View className="h-11 min-h-11 flex-row items-center justify-end">
        {Platform.OS === "ios" ? (
          <Host matchContents ignoreSafeArea="all">
            <Button
              variant="text"
              onPress={handleClose}
              modifiers={[
                buttonStyle("glass"),
                buttonBorderShape("circle"),
                controlSize("large"),
                foregroundStyle(colors.foreground),
              ]}
            >
              <Icon name={CLOSE_ICON} color={colors.foreground} size={16} />
            </Button>
          </Host>
        ) : (
          <Pressable
            onPress={handleClose}
            hitSlop={8}
            className="size-11 items-center justify-center rounded-full bg-card border border-border/40 active:bg-accent"
          >
            <X size={20} color={colors.foreground} strokeWidth={2} />
          </Pressable>
        )}
      </View>

      <Text className="mb-1 mt-2 text-[34px] font-bold tracking-tight text-foreground">
        Settings
      </Text>
    </View>
  )
})
