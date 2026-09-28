import BottomSheet, {
  BottomSheetBackdrop,
  type BottomSheetBackdropProps,
  BottomSheetView,
} from "@gorhom/bottom-sheet"
import type { RefObject } from "react"
import { forwardRef, useCallback, useImperativeHandle, useMemo, useRef } from "react"
import { Pressable, ScrollView, Text, View } from "react-native"
import type {
  EnrichedMarkdownTextInputInstance,
  HeadingLevel,
  StyleState,
} from "react-native-enriched-markdown"
import { useAppTheme } from "@/hooks/useAppTheme"
import { ToolbarIcon, type ToolbarIconName } from "./ToolbarIcon"

const STYLES = [
  {
    id: "title",
    label: "Title",
    fontSize: 22,
    fontWeight: "700" as const,
    letterSpacing: -0.5,
  },
  {
    id: "heading",
    label: "Heading",
    fontSize: 18,
    fontWeight: "600" as const,
    letterSpacing: -0.3,
  },
  {
    id: "subheading",
    label: "Subheading",
    fontSize: 16,
    fontWeight: "600" as const,
  },
  {
    id: "body",
    label: "Body",
    fontSize: 15,
    fontWeight: "400" as const,
  },
] as const

type HeadingType = (typeof STYLES)[number]["id"]

const HEADING_LEVELS: Record<Exclude<HeadingType, "body">, HeadingLevel> = {
  title: 1,
  heading: 2,
  subheading: 3,
}

export interface FormatSheetRef {
  open: () => void
  close: () => void
}

interface FormatSheetProps {
  inputRef: RefObject<EnrichedMarkdownTextInputInstance | null>
  styleState: StyleState | null
  onClose?: () => void
}

type FormatButtonProps = {
  icon: ToolbarIconName
  active?: boolean
  disabled?: boolean
  onPress: () => void
}

function FormatButton({
  icon,
  active = false,
  disabled: isDisabled = false,
  onPress,
}: FormatButtonProps) {
  const { colors, isDarkMode } = useAppTheme()
  const iconColor = active
    ? isDarkMode
      ? "#32285F"
      : "#FFFFFF"
    : isDisabled
      ? colors.mutedForeground
      : colors.foreground

  return (
    <Pressable
      onPress={onPress}
      disabled={isDisabled}
      hitSlop={4}
      className={`h-10.5 flex-1 items-center justify-center rounded-xl transition-all active:opacity-60 ${
        active ? "bg-primary" : "bg-transparent"
      } ${isDisabled ? "opacity-35" : "opacity-100"}`}
    >
      <ToolbarIcon name={icon} size={20} color={iconColor} />
    </Pressable>
  )
}

export const FormatSheet = forwardRef<FormatSheetRef, FormatSheetProps>(function FormatSheet(
  { inputRef, styleState, onClose },
  ref,
) {
  const bottomSheetRef = useRef<BottomSheet>(null)
  const { colors, isDarkMode } = useAppTheme()
  const snapPoints = useMemo(() => [265], [])

  const open = useCallback(() => {
    bottomSheetRef.current?.snapToIndex(0)
  }, [])

  const close = useCallback(() => {
    bottomSheetRef.current?.close()
  }, [])

  useImperativeHandle(ref, () => ({ open, close }), [open, close])

  const focusInput = useCallback(() => {
    try {
      inputRef.current?.focus()
    } catch {}
  }, [inputRef])

  const handleManualClose = useCallback(() => {
    bottomSheetRef.current?.close()
    focusInput()
    onClose?.()
  }, [focusInput, onClose])

  const headingLevel = styleState?.heading.isActive ? styleState.heading.level : 0
  const activeHeading: HeadingLevel | null = headingLevel === 0 ? null : headingLevel

  let activeStyle: HeadingType = "body"
  if (headingLevel === 1) activeStyle = "title"
  else if (headingLevel === 2) activeStyle = "heading"
  else if (headingLevel === 3) activeStyle = "subheading"

  const handleSelectStyle = (type: HeadingType) => {
    if (type === "body") {
      if (activeHeading) {
        inputRef.current?.toggleHeading(activeHeading)
      }
      return
    }
    const level = HEADING_LEVELS[type]
    if (headingLevel === level) {
      inputRef.current?.toggleHeading(level)
    } else {
      if (activeHeading) {
        inputRef.current?.toggleHeading(activeHeading)
      }
      inputRef.current?.toggleHeading(level)
    }
  }

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

  const handleSheetChange = useCallback(
    (index: number) => {
      if (index === -1) {
        focusInput()
        onClose?.()
      }
    },
    [focusInput, onClose],
  )

  return (
    <BottomSheet
      ref={bottomSheetRef}
      index={-1}
      snapPoints={snapPoints}
      enableDynamicSizing={false}
      enablePanDownToClose={true}
      onChange={handleSheetChange}
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
      <BottomSheetView className="w-full flex-1 px-4.5 pt-1 pb-6">
        <View className="mb-3 flex-row items-center justify-between">
          <Text className="text-[19px] font-bold text-foreground">Format</Text>

          <Pressable
            onPress={handleManualClose}
            hitSlop={8}
            className="size-7 items-center justify-center rounded-full bg-accent active:opacity-60"
          >
            <ToolbarIcon name="close" size={13} color={colors.foreground} />
          </Pressable>
        </View>

        <ScrollView
          horizontal
          showsHorizontalScrollIndicator={false}
          bounces={false}
          contentContainerStyle={{
            gap: 8,
            paddingRight: 16,
            alignItems: "center",
          }}
          className="mb-3"
        >
          {STYLES.map((style) => {
            const isSelected = activeStyle === style.id

            return (
              <Pressable
                key={style.id}
                onPress={() => handleSelectStyle(style.id)}
                className={`h-9 items-center justify-center rounded-full px-4 active:opacity-75 ${
                  isSelected ? "bg-primary" : "bg-accent"
                }`}
              >
                <Text
                  style={{
                    fontSize: style.fontSize,
                    fontWeight: style.fontWeight,
                    letterSpacing: "letterSpacing" in style ? style.letterSpacing : undefined,
                  }}
                  className={
                    isSelected ? (isDarkMode ? "text-[#32285F]" : "text-white") : "text-foreground"
                  }
                >
                  {style.label}
                </Text>
              </Pressable>
            )
          })}
        </ScrollView>

        <View className="mb-2.5 w-full flex-row overflow-hidden rounded-2xl bg-background border border-border/40 p-1">
          <FormatButton
            icon="bold"
            active={styleState?.bold.isActive ?? false}
            onPress={() => inputRef.current?.toggleBold()}
          />
          <FormatButton
            icon="italic"
            active={styleState?.italic.isActive ?? false}
            onPress={() => inputRef.current?.toggleItalic()}
          />
          <FormatButton
            icon="underline"
            active={styleState?.underline.isActive ?? false}
            onPress={() => inputRef.current?.toggleUnderline()}
          />
          <FormatButton
            icon="strike"
            active={styleState?.strikethrough.isActive ?? false}
            onPress={() => inputRef.current?.toggleStrikethrough()}
          />
        </View>

        <View className="w-full flex-row overflow-hidden rounded-2xl bg-background border border-border/40 p-1">
          <FormatButton
            icon="bulletList"
            active={styleState?.unorderedList.isActive ?? false}
            onPress={() => inputRef.current?.toggleUnorderedList()}
          />
          <FormatButton
            icon="orderedList"
            active={styleState?.orderedList.isActive ?? false}
            onPress={() => inputRef.current?.toggleOrderedList()}
          />
          <FormatButton icon="outdent" onPress={() => inputRef.current?.outdentList()} />
          <FormatButton icon="indent" onPress={() => inputRef.current?.indentList()} />
        </View>
      </BottomSheetView>
    </BottomSheet>
  )
})
