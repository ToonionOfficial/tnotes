import {
  BottomSheetBackdrop,
  type BottomSheetBackdropProps,
  BottomSheetModal,
  BottomSheetScrollView,
} from "@gorhom/bottom-sheet"
import * as Haptics from "expo-haptics"
import { Check, Folder as FolderOutline } from "lucide-react-native"
import { forwardRef, useCallback, useImperativeHandle, useMemo, useRef } from "react"
import { Platform, Pressable, Text, View } from "react-native"
import type { SearchResult } from "@/db/queries"
import type { Note } from "@/db/schema"
import { useInfiniteFolders } from "@/hooks/useFolders"
import { FolderIcon } from "./FolderIcon"

interface MoveNoteSheetProps {
  note: Note | SearchResult | null
  onSelectFolder: (targetFolderId: string | null) => void
}

export interface MoveNoteSheetRef {
  open: () => void
  close: () => void
}

export const MoveNoteSheet = forwardRef<MoveNoteSheetRef, MoveNoteSheetProps>(
  function MoveNoteSheet({ note, onSelectFolder }, ref) {
    const bottomSheetModalRef = useRef<BottomSheetModal>(null)
    const { data: folderPages } = useInfiniteFolders()
    const allFolders = folderPages?.pages.flatMap((page) => page.folders) ?? []

    const snapPoints = useMemo(() => ["65%"], [])

    const open = useCallback(() => {
      void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
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

    if (!note) return null

    const currentFolderId = note.folderId ?? null

    const handleSelect = (folderId: string | null) => {
      void Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success)
      close()
      onSelectFolder(folderId)
    }

    return (
      <BottomSheetModal
        ref={bottomSheetModalRef}
        snapPoints={snapPoints}
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
        <BottomSheetScrollView
          contentContainerStyle={{
            paddingHorizontal: 20,
            paddingTop: 8,
            paddingBottom: 40,
          }}
          showsVerticalScrollIndicator={false}
        >
          <Text className="mb-3 text-[17px] font-bold text-foreground">Move to Folder</Text>

          <View className="overflow-hidden">
            {/* Root "Notes" (no folder) option */}
            <Pressable
              onPress={() => handleSelect(null)}
              className="flex-row items-center justify-between py-3.5 active:opacity-60"
            >
              <View className="flex-row items-center gap-3">
                <FolderOutline size={20} color="#CABEFF" />
                <Text className="text-[15px] font-medium text-white">Notes (No folder)</Text>
              </View>
              {currentFolderId === null && <Check size={18} color="#CABEFF" strokeWidth={2.5} />}
            </Pressable>

            {allFolders.map((folder) => {
              const isSelected = currentFolderId === folder.id
              return (
                <View key={folder.id}>
                  <View className="ml-8 h-[0.5px] bg-white/10" />
                  <Pressable
                    onPress={() => handleSelect(folder.id)}
                    className="flex-row items-center justify-between py-3.5 active:opacity-60"
                  >
                    <View className="flex-row items-center gap-3">
                      <FolderIcon name={folder.icon} size={20} />
                      <Text
                        numberOfLines={1}
                        className="text-[15px] font-medium text-white max-w-[220px]"
                      >
                        {folder.name}
                      </Text>
                    </View>
                    {isSelected && <Check size={18} color="#CABEFF" strokeWidth={2.5} />}
                  </Pressable>
                </View>
              )
            })}
          </View>
        </BottomSheetScrollView>
      </BottomSheetModal>
    )
  },
)
