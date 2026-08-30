import {
  BottomSheetBackdrop,
  type BottomSheetBackdropProps,
  BottomSheetModal,
  BottomSheetView,
} from "@gorhom/bottom-sheet"
import * as Haptics from "expo-haptics"
import { Pencil, Trash2 } from "lucide-react-native"
import { forwardRef, useCallback, useImperativeHandle, useRef, useState } from "react"
import { Pressable, Text, View } from "react-native"
import type { Folder } from "@/db/schema"
import { useAppTheme } from "@/hooks/useAppTheme"
import { DEFAULT_FOLDER_ICON, FolderIcon } from "./FolderIcon"

interface FolderActionSheetProps {
  onEdit?: (folder: Folder) => void
  onDelete?: (folder: Folder) => void
  getFolderCount?: (folderId: string | null) => number
}

export interface FolderActionSheetRef {
  open: (folder: Folder) => void
  close: () => void
}

export const FolderActionSheet = forwardRef<FolderActionSheetRef, FolderActionSheetProps>(
  function FolderActionSheet({ onEdit, onDelete, getFolderCount }, ref) {
    const bottomSheetModalRef = useRef<BottomSheetModal>(null)
    const [activeFolder, setActiveFolder] = useState<Folder | null>(null)
    const { colors, isDarkMode } = useAppTheme()

    const open = useCallback((folder: Folder) => {
      setActiveFolder(folder)
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

    const noteCount = activeFolder && getFolderCount ? getFolderCount(activeFolder.id) : 0

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
        {activeFolder ? (
          <BottomSheetView className="px-5 pb-8 pt-2">
            {/* Folder Preview Header */}
            <View className="mb-3 px-1">
              <View className="flex-row items-center gap-2.5">
                <View className="size-8 items-center justify-center">
                  <FolderIcon
                    name={activeFolder.icon || DEFAULT_FOLDER_ICON}
                    size={20}
                    color={colors.primary}
                    fill={colors.primary}
                  />
                </View>
                <View className="flex-1">
                  <Text numberOfLines={1} className="text-[17px] font-semibold text-foreground">
                    {activeFolder.name}
                  </Text>
                  <Text className="text-[12px] text-muted-foreground/60">
                    {noteCount} {noteCount === 1 ? "note" : "notes"}
                  </Text>
                </View>
              </View>
            </View>

            {/* Action Items List */}
            <View className="overflow-hidden">
              {onEdit && (
                <Pressable
                  onPress={() => {
                    close()
                    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
                    onEdit(activeFolder)
                  }}
                  className="flex-row items-center justify-between py-3.5 active:opacity-60"
                >
                  <View className="flex-row items-center gap-3">
                    <Pencil size={19} color={colors.foreground} />
                    <Text className="text-[15px] font-medium text-foreground">Edit Folder</Text>
                  </View>
                </Pressable>
              )}

              {onEdit && <View className="ml-8 h-[0.5px] bg-border" />}

              {onDelete && (
                <Pressable
                  onPress={() => {
                    close()
                    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
                    onDelete(activeFolder)
                  }}
                  className="flex-row items-center justify-between py-3.5 active:opacity-60"
                >
                  <View className="flex-row items-center gap-3">
                    <Trash2 size={19} color="#FF6B6B" />
                    <Text className="text-[15px] font-medium text-[#FF6B6B]">Delete Folder</Text>
                  </View>
                </Pressable>
              )}
            </View>
          </BottomSheetView>
        ) : (
          <BottomSheetView>
            <View className="h-4" />
          </BottomSheetView>
        )}
      </BottomSheetModal>
    )
  },
)
