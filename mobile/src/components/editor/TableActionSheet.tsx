import type { EditorBridge } from "@10play/tentap-editor"
import BottomSheet, {
  BottomSheetBackdrop,
  type BottomSheetBackdropProps,
  BottomSheetView,
} from "@gorhom/bottom-sheet"
import * as Haptics from "expo-haptics"
import {
  BetweenHorizontalEnd,
  BetweenVerticalEnd,
  Columns3,
  Rows3,
  Table as TableIcon,
  Trash2,
  X,
} from "lucide-react-native"
import { forwardRef, useCallback, useImperativeHandle, useMemo, useRef } from "react"
import { Platform, Pressable, Text, View } from "react-native"
import {
  addTableColumnLeft,
  addTableColumnRight,
  addTableRowAbove,
  addTableRowBelow,
  deleteTable,
  deleteTableColumn,
  deleteTableRow,
  insertTable,
} from "./tableHelper"

export interface TableActionSheetRef {
  open: () => void
  close: () => void
}

interface TableActionSheetProps {
  editor: EditorBridge
  onClose?: () => void
}

export const TableActionSheet = forwardRef<TableActionSheetRef, TableActionSheetProps>(
  function TableActionSheet({ editor, onClose }, ref) {
    const bottomSheetRef = useRef<BottomSheet>(null)
    const snapPoints = useMemo(() => ["50%"], [])

    const open = useCallback(() => {
      void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
      bottomSheetRef.current?.snapToIndex(0)
    }, [])

    const close = useCallback(() => {
      bottomSheetRef.current?.close()
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

    const handleInsert = (rows: number, cols: number) => {
      insertTable(editor, rows, cols)
      close()
      onClose?.()
    }

    const handleAddRow = (above = false) => {
      if (above) {
        addTableRowAbove(editor)
      } else {
        addTableRowBelow(editor)
      }
      close()
      onClose?.()
    }

    const handleAddColumn = (left = false) => {
      if (left) {
        addTableColumnLeft(editor)
      } else {
        addTableColumnRight(editor)
      }
      close()
      onClose?.()
    }

    const handleDeleteRow = () => {
      deleteTableRow(editor)
      close()
      onClose?.()
    }

    const handleDeleteColumn = () => {
      deleteTableColumn(editor)
      close()
      onClose?.()
    }

    const handleDeleteTable = () => {
      deleteTable(editor)
      close()
      onClose?.()
    }

    return (
      <BottomSheet
        ref={bottomSheetRef}
        index={-1}
        snapPoints={snapPoints}
        enableDynamicSizing={false}
        enablePanDownToClose
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
        <BottomSheetView className="flex-1 px-5 pt-1 pb-6">
          <View className="mb-4 flex-row items-center justify-between">
            <View className="flex-row items-center gap-2">
              <TableIcon size={20} color="#CABEFF" />
              <Text className="text-[18px] font-bold text-foreground">Table Options</Text>
            </View>

            <Pressable
              onPress={close}
              hitSlop={8}
              className="size-7 items-center justify-center rounded-full bg-white/10 active:opacity-60"
            >
              <X size={14} color="#E6E1E9" />
            </Pressable>
          </View>

          {/* Quick Insert Table Presets */}
          <Text className="mb-2 text-[12px] font-semibold uppercase tracking-wider text-white/50">
            Insert New Table
          </Text>
          <View className="mb-4 flex-row gap-2.5">
            {[
              { label: "2 × 2", r: 2, c: 2 },
              { label: "3 × 3", r: 3, c: 3 },
              { label: "4 × 4", r: 4, c: 4 },
            ].map((preset) => (
              <Pressable
                key={preset.label}
                onPress={() => handleInsert(preset.r, preset.c)}
                className="flex-1 items-center justify-center rounded-xl bg-white/[0.08] py-2.5 active:bg-primary/20"
              >
                <Text className="text-[14px] font-semibold text-[#CABEFF]">{preset.label}</Text>
              </Pressable>
            ))}
          </View>

          {/* Modify Table Section */}
          <Text className="mb-2 text-[12px] font-semibold uppercase tracking-wider text-white/50">
            Modify Current Table
          </Text>
          <View className="gap-2">
            <View className="flex-row gap-2">
              <Pressable
                onPress={() => handleAddRow(false)}
                className="flex-1 flex-row items-center justify-center gap-2 rounded-xl bg-white/[0.06] py-2.5 active:opacity-60"
              >
                <Rows3 size={16} color="#CABEFF" />
                <Text className="text-[13px] font-medium text-white">+ Add Row</Text>
              </Pressable>

              <Pressable
                onPress={() => handleAddColumn(false)}
                className="flex-1 flex-row items-center justify-center gap-2 rounded-xl bg-white/[0.06] py-2.5 active:opacity-60"
              >
                <Columns3 size={16} color="#CABEFF" />
                <Text className="text-[13px] font-medium text-white">+ Add Column</Text>
              </Pressable>
            </View>

            <View className="flex-row gap-2">
              <Pressable
                onPress={handleDeleteRow}
                className="flex-1 flex-row items-center justify-center gap-2 rounded-xl bg-white/[0.06] py-2.5 active:opacity-60"
              >
                <BetweenHorizontalEnd size={16} color="#FF9F9B" />
                <Text className="text-[13px] font-medium text-red-300">Delete Row</Text>
              </Pressable>

              <Pressable
                onPress={handleDeleteColumn}
                className="flex-1 flex-row items-center justify-center gap-2 rounded-xl bg-white/[0.06] py-2.5 active:opacity-60"
              >
                <BetweenVerticalEnd size={16} color="#FF9F9B" />
                <Text className="text-[13px] font-medium text-red-300">Delete Column</Text>
              </Pressable>
            </View>

            <Pressable
              onPress={handleDeleteTable}
              className="flex-row items-center justify-center gap-2 rounded-xl bg-red-500/10 py-2.5 active:opacity-60"
            >
              <Trash2 size={16} color="#FF453A" />
              <Text className="text-[14px] font-semibold text-red-400">Delete Entire Table</Text>
            </Pressable>
          </View>
        </BottomSheetView>
      </BottomSheet>
    )
  },
)
