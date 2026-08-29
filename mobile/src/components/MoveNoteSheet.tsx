import {
  BottomSheetBackdrop,
  type BottomSheetBackdropProps,
  BottomSheetModal,
  BottomSheetScrollView,
} from "@gorhom/bottom-sheet"
import * as Haptics from "expo-haptics"
import { Check, CornerDownRight, Folder as FolderOutline } from "lucide-react-native"
import { forwardRef, useCallback, useImperativeHandle, useMemo, useRef, useState } from "react"
import { Platform, Pressable, Text, View } from "react-native"
import type { SearchResult } from "@/db/queries"
import type { Note } from "@/db/schema"
import { useInfiniteFolders } from "@/hooks/useFolders"
import { buildFolderTree } from "@/utils/folderTree"
import { DEFAULT_FOLDER_ICON, FolderIcon } from "./FolderIcon"

interface MoveNoteSheetProps {
  note?: Note | SearchResult | null
  onSelectFolder: (targetFolderId: string | null) => void
}

export interface MoveNoteSheetRef {
  open: (note?: Note | SearchResult) => void
  close: () => void
}

export const MoveNoteSheet = forwardRef<MoveNoteSheetRef, MoveNoteSheetProps>(
  function MoveNoteSheet({ note: propNote, onSelectFolder }, ref) {
    const bottomSheetModalRef = useRef<BottomSheetModal>(null)
    const [internalNote, setInternalNote] = useState<Note | SearchResult | null>(propNote ?? null)

    const activeNote = propNote ?? internalNote

    const { data: folderPages } = useInfiniteFolders()
    const allFolders = useMemo(
      () => folderPages?.pages.flatMap((page) => page.folders) ?? [],
      [folderPages],
    )

    const treeFolders = useMemo(() => buildFolderTree(allFolders), [allFolders])
    const snapPoints = useMemo(() => ["65%"], [])

    const open = useCallback((noteToOpen?: Note | SearchResult) => {
      if (noteToOpen) {
        setInternalNote(noteToOpen)
      }
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

    const currentFolderId = activeNote?.folderId ?? null

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
            {/* Root "All Notes" (no folder) option */}
            <Pressable
              onPress={() => handleSelect(null)}
              className="flex-row items-center justify-between py-3.5 active:opacity-60"
            >
              <View className="flex-1 flex-row items-center gap-3">
                <View className="size-8 items-center justify-center rounded-lg bg-white/[0.08]">
                  <FolderOutline size={18} color="#CABEFF" />
                </View>
                <Text
                  numberOfLines={1}
                  className={`text-[15px] font-medium ${
                    currentFolderId === null ? "font-semibold text-[#CABEFF]" : "text-white"
                  }`}
                >
                  All Notes (No folder)
                </Text>
              </View>
              {currentFolderId === null && <Check size={18} color="#CABEFF" strokeWidth={2.5} />}
            </Pressable>

            {treeFolders.map(({ folder, depth }) => {
              const isSelected = currentFolderId === folder.id
              const indentPadding = depth * 22

              return (
                <View key={folder.id}>
                  <View
                    style={{ marginLeft: 36 + indentPadding }}
                    className="h-[0.5px] bg-white/10"
                  />
                  <Pressable
                    onPress={() => handleSelect(folder.id)}
                    style={{ paddingLeft: indentPadding }}
                    className="flex-row items-center justify-between py-3.5 active:opacity-60"
                  >
                    <View className="flex-1 flex-row items-center gap-2.5">
                      {depth > 0 && <CornerDownRight size={14} color="#8E8C99" strokeWidth={2} />}
                      <View className="size-8 items-center justify-center rounded-lg bg-white/[0.08]">
                        <FolderIcon
                          name={folder.icon || DEFAULT_FOLDER_ICON}
                          size={18}
                          color="#CABEFF"
                          fill="#CABEFF"
                        />
                      </View>
                      <Text
                        numberOfLines={1}
                        className={`flex-1 text-[15px] font-medium ${
                          isSelected ? "font-semibold text-[#CABEFF]" : "text-white"
                        }`}
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
