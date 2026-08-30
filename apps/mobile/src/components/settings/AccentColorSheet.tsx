import {
  BottomSheetBackdrop,
  type BottomSheetBackdropProps,
  BottomSheetModal,
  BottomSheetView,
} from "@gorhom/bottom-sheet"
import * as Haptics from "expo-haptics"
import { Check } from "lucide-react-native"
import { forwardRef, useCallback, useImperativeHandle, useRef } from "react"
import { Pressable, Text, View } from "react-native"
import { ACCENT_COLOR_PRESETS, type AccentColorId } from "@/constants/accentColors"
import { useAppTheme } from "@/hooks/useAppTheme"

export interface AccentColorSheetRef {
  open: () => void
  close: () => void
}

export const AccentColorSheet = forwardRef<AccentColorSheetRef>(function AccentColorSheet(_, ref) {
  const bottomSheetModalRef = useRef<BottomSheetModal>(null)
  const { accentColor, setAccentColor, colors, isDarkMode } = useAppTheme()

  const open = useCallback(() => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
    bottomSheetModalRef.current?.present()
  }, [])

  const close = useCallback(() => {
    bottomSheetModalRef.current?.dismiss()
  }, [])

  useImperativeHandle(ref, () => ({ open, close }), [open, close])

  const renderBackdrop = useCallback(
    (props: BottomSheetBackdropProps) => (
      <BottomSheetBackdrop
        {...props}
        appearsOnIndex={0}
        disappearsOnIndex={-1}
        opacity={0.45}
        pressBehavior="close"
      />
    ),
    [],
  )

  const handleSelectColor = (id: AccentColorId) => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    setAccentColor(id)
  }

  return (
    <BottomSheetModal
      ref={bottomSheetModalRef}
      enableDynamicSizing
      enablePanDownToClose
      backdropComponent={renderBackdrop}
      handleIndicatorStyle={{
        backgroundColor: isDarkMode ? "rgba(255, 255, 255, 0.25)" : "rgba(0, 0, 0, 0.2)",
        width: 36,
        height: 4,
      }}
      backgroundStyle={{
        backgroundColor: colors.card,
        borderTopLeftRadius: 28,
        borderTopRightRadius: 28,
        borderWidth: 1,
        borderColor: colors.border,
      }}
    >
      <BottomSheetView className="px-5 pb-9 pt-2">
        <View className="mb-4 px-1">
          <Text className="text-[19px] font-bold text-foreground">Accent Color</Text>
          <Text className="mt-0.5 text-[13px] text-muted-foreground">
            Choose an accent color for highlights and badges
          </Text>
        </View>

        <View className="overflow-hidden rounded-3xl bg-background border border-border/40">
          {ACCENT_COLOR_PRESETS.map((preset, index) => {
            const isSelected = accentColor === preset.id
            const isFirst = index === 0

            return (
              <View key={preset.id}>
                {!isFirst && <View className="ml-14 h-[0.5px] bg-border" />}
                <Pressable
                  onPress={() => handleSelectColor(preset.id)}
                  className="flex-row items-center justify-between px-4 py-3.5 active:bg-accent"
                >
                  <View className="flex-row items-center gap-3.5">
                    <View
                      style={{ backgroundColor: preset.preview }}
                      className="size-7 rounded-full shadow-sm"
                    />
                    <Text className="text-[16px] font-medium text-foreground">{preset.label}</Text>
                  </View>

                  {isSelected && <Check size={20} color={colors.primary} strokeWidth={2.5} />}
                </Pressable>
              </View>
            )
          })}
        </View>
      </BottomSheetView>
    </BottomSheetModal>
  )
})
