import type { EditorBridge } from "@10play/tentap-editor"
import { useBridgeState } from "@10play/tentap-editor"
import BottomSheet, {
  BottomSheetBackdrop,
  type BottomSheetBackdropProps,
  BottomSheetView,
} from "@gorhom/bottom-sheet"
import { useCallback, useEffect, useMemo, useRef } from "react"
import { Platform, Pressable, ScrollView, Text, View } from "react-native"
import { ToolbarIcon, type ToolbarIconName } from "./ToolbarIcon"

const STYLES = [
  { id: "title", label: "Title" },
  { id: "heading", label: "Heading" },
  { id: "subheading", label: "Subheading" },
  { id: "body", label: "Body" },
  { id: "code", label: "Monospaced" },
  { id: "quote", label: "Quote" },
] as const

type HeadingType = (typeof STYLES)[number]["id"]

interface FormatSheetProps {
  editor: EditorBridge
  isOpen: boolean
  onClose: () => void
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
  return (
    <Pressable
      onPress={onPress}
      disabled={isDisabled}
      hitSlop={4}
      className={`h-10.5 flex-1 items-center justify-center rounded-xl transition-all active:opacity-60 ${
        active ? "bg-primary" : "bg-transparent"
      } ${isDisabled ? "opacity-35" : "opacity-100"}`}
    >
      <ToolbarIcon name={icon} size={20} color={active ? "#32285F" : "#E6E1E9"} />
    </Pressable>
  )
}

export function FormatSheet({ editor, isOpen, onClose }: FormatSheetProps) {
  const bottomSheetRef = useRef<BottomSheet>(null)
  const editorState = useBridgeState(editor)
  const snapPoints = useMemo(() => [260], [])

  useEffect(() => {
    if (isOpen) {
      bottomSheetRef.current?.snapToIndex(0)
    } else {
      bottomSheetRef.current?.close()
    }
  }, [isOpen])

  const handleManualClose = useCallback(() => {
    bottomSheetRef.current?.close()
    try {
      editor.focus()
    } catch {}
    onClose()
  }, [editor, onClose])

  const currentHeadingLevel = editorState.headingLevel
  const isBlockquote = editorState.isBlockquoteActive
  const isCodeActive = editorState.isCodeActive

  let activeStyle: HeadingType = "body"
  if (currentHeadingLevel === 1) activeStyle = "title"
  else if (currentHeadingLevel === 2) activeStyle = "heading"
  else if (currentHeadingLevel === 3) activeStyle = "subheading"
  else if (isBlockquote) activeStyle = "quote"
  else if (isCodeActive) activeStyle = "code"

  const handleSelectStyle = (type: HeadingType) => {
    switch (type) {
      case "title":
        editor.toggleHeading(1)
        break
      case "heading":
        editor.toggleHeading(2)
        break
      case "subheading":
        editor.toggleHeading(3)
        break
      case "body":
        if (currentHeadingLevel) editor.toggleHeading(currentHeadingLevel as 1 | 2 | 3)
        if (isBlockquote) editor.toggleBlockquote()
        break
      case "quote":
        editor.toggleBlockquote()
        break
      case "code":
        editor.toggleCode()
        break
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
        if (isOpen) {
          try {
            editor.focus()
          } catch {}
          onClose()
        }
      }
    },
    [isOpen, onClose, editor],
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
        backgroundColor: "rgba(255, 255, 255, 0.25)",
        width: 36,
        height: 4,
      }}
      backgroundStyle={{
        backgroundColor: Platform.select({
          ios: "#1C1B20",
          default: "#1C1B20",
        }),
        borderTopLeftRadius: 28,
        borderTopRightRadius: 28,
        borderWidth: 1,
        borderColor: "rgba(255, 255, 255, 0.1)",
      }}
    >
      <BottomSheetView className="w-full flex-1 px-4.5 pt-1 pb-6">
        <View className="mb-3 flex-row items-center justify-between">
          <Text className="text-[19px] font-bold text-foreground">Format</Text>

          <Pressable
            onPress={handleManualClose}
            hitSlop={8}
            className="size-7 items-center justify-center rounded-full bg-white/10 active:opacity-60"
          >
            <ToolbarIcon name="close" size={13} color="#E6E1E9" />
          </Pressable>
        </View>

        <ScrollView
          horizontal
          showsHorizontalScrollIndicator={false}
          bounces={false}
          contentContainerStyle={{
            gap: 8,
            paddingRight: 16,
          }}
          className="mb-3"
        >
          {STYLES.map((style) => {
            const isSelected = activeStyle === style.id

            return (
              <Pressable
                key={style.id}
                onPress={() => handleSelectStyle(style.id)}
                className={`h-8.5 items-center justify-center rounded-full px-4 active:opacity-75 ${
                  isSelected ? "bg-primary" : "bg-white/6"
                }`}
              >
                <Text
                  className={`text-[14px] font-semibold ${
                    isSelected ? "text-[#32285F]" : "text-foreground"
                  }`}
                >
                  {style.label}
                </Text>
              </Pressable>
            )
          })}
        </ScrollView>

        <View className="mb-2.5 w-full flex-row overflow-hidden rounded-2xl bg-[#28272E] p-1">
          <FormatButton
            icon="bold"
            active={editorState.isBoldActive}
            disabled={!editorState.canToggleBold}
            onPress={() => editor.toggleBold()}
          />
          <FormatButton
            icon="italic"
            active={editorState.isItalicActive}
            disabled={!editorState.canToggleItalic}
            onPress={() => editor.toggleItalic()}
          />
          <FormatButton
            icon="underline"
            active={editorState.isUnderlineActive}
            disabled={!editorState.canToggleUnderline}
            onPress={() => editor.toggleUnderline()}
          />
          <FormatButton
            icon="strike"
            active={editorState.isStrikeActive}
            disabled={!editorState.canToggleStrike}
            onPress={() => editor.toggleStrike()}
          />
          <FormatButton
            icon="code"
            active={editorState.isCodeActive}
            disabled={!editorState.canToggleCode}
            onPress={() => editor.toggleCode()}
          />
        </View>

        <View className="w-full flex-row overflow-hidden rounded-2xl bg-[#28272E] p-1">
          <FormatButton
            icon="checklist"
            active={editorState.isTaskListActive}
            disabled={!editorState.canToggleTaskList}
            onPress={() => editor.toggleTaskList()}
          />
          <FormatButton
            icon="bulletList"
            active={editorState.isBulletListActive}
            disabled={!editorState.canToggleBulletList}
            onPress={() => editor.toggleBulletList()}
          />
          <FormatButton
            icon="orderedList"
            active={editorState.isOrderedListActive}
            disabled={!editorState.canToggleOrderedList}
            onPress={() => editor.toggleOrderedList()}
          />
          <FormatButton
            icon="outdent"
            disabled={!editorState.canLift && !editorState.canLiftTaskListItem}
            onPress={() => (editorState.canLift ? editor.lift() : editor.liftTaskListItem())}
          />
          <FormatButton
            icon="indent"
            disabled={!editorState.canSink && !editorState.canSinkTaskListItem}
            onPress={() => (editorState.canSink ? editor.sink() : editor.sinkTaskListItem())}
          />
        </View>
      </BottomSheetView>
    </BottomSheet>
  )
}
