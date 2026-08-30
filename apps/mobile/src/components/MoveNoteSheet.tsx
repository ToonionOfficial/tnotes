import {
  BottomSheetBackdrop,
  type BottomSheetBackdropProps,
  BottomSheetModal,
  BottomSheetScrollView,
} from "@gorhom/bottom-sheet"
import * as Haptics from "expo-haptics"
import {
  Check,
  ChevronDown,
  ChevronRight,
  CornerDownRight,
  Folder as FolderOutline,
} from "lucide-react-native"
import { forwardRef, useCallback, useImperativeHandle, useMemo, useRef, useState } from "react"
import { Pressable, Text, View } from "react-native"
import { DEFAULT_FOLDER_ICON, FolderIcon } from "@/components/FolderIcon"
import type { SearchResult } from "@/db/queries"
import type { Folder, Note } from "@/db/schema"
import { useAppTheme } from "@/hooks/useAppTheme"
import { useInfiniteFolders } from "@/hooks/useFolders"
import { buildFolderTree } from "@/utils/folderTree"

interface MoveNoteSheetProps {
  note?: Note | SearchResult | null
  onSelectFolder: (folderId: string | null) => void
}

export interface MoveNoteSheetRef {
  open: (noteToMove?: Note | SearchResult) => void
  close: () => void
}

export const MoveNoteSheet = forwardRef<MoveNoteSheetRef, MoveNoteSheetProps>(
  function MoveNoteSheet({ note: propNote, onSelectFolder }, ref) {
    const bottomSheetModalRef = useRef<BottomSheetModal>(null)
    const [internalNote, setInternalNote] = useState<Note | SearchResult | null>(null)
    const activeNote = internalNote || propNote
    const { colors, isDarkMode } = useAppTheme()

    const { data: folderPages } = useInfiniteFolders()
    const allFolders = useMemo(
      () => folderPages?.pages.flatMap((page) => page.folders) ?? [],
      [folderPages],
    )
    const [expandedFolderIds, setExpandedFolderIds] = useState<Set<string>>(new Set())

    const snapPoints = useMemo(() => ["60%"], [])

    const folderMap = useMemo(() => {
      return new Map<string, Folder>(allFolders.map((f) => [f.id, f]))
    }, [allFolders])

    const treeFolders = useMemo(
      () => buildFolderTree(allFolders, expandedFolderIds),
      [allFolders, expandedFolderIds],
    )

    const toggleExpand = useCallback((folderId: string) => {
      void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
      setExpandedFolderIds((prev) => {
        const next = new Set(prev)
        if (next.has(folderId)) {
          next.delete(folderId)
        } else {
          next.add(folderId)
        }
        return next
      })
    }, [])

    const open = useCallback(
      (noteToMove?: Note | SearchResult) => {
        const targetNote = noteToMove || propNote
        if (targetNote) {
          setInternalNote(targetNote)
        }

        if (targetNote?.folderId) {
          const set = new Set<string>()
          let current = folderMap.get(targetNote.folderId)
          while (current?.parentId) {
            set.add(current.parentId)
            current = folderMap.get(current.parentId)
          }
          setExpandedFolderIds(set)
        } else {
          setExpandedFolderIds(new Set())
        }

        void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
        bottomSheetModalRef.current?.present()
      },
      [folderMap, propNote],
    )

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
                <View className="size-8 items-center justify-center rounded-lg bg-accent">
                  <FolderOutline size={18} color={colors.primary} />
                </View>
                <Text
                  numberOfLines={1}
                  className={`text-[15px] font-medium ${
                    currentFolderId === null ? "font-semibold text-primary" : "text-foreground"
                  }`}
                >
                  All Notes (No folder)
                </Text>
              </View>
              {currentFolderId === null && (
                <Check size={18} color={colors.primary} strokeWidth={2.5} />
              )}
            </Pressable>

            {treeFolders.map(({ folder, depth, hasChildren, isCollapsed }) => {
              const isSelected = currentFolderId === folder.id
              const indentPadding = depth * 20

              return (
                <View key={folder.id}>
                  <View
                    style={{ marginLeft: 36 + indentPadding }}
                    className="h-[0.5px] bg-border"
                  />
                  <Pressable
                    onPress={() => handleSelect(folder.id)}
                    style={{ paddingLeft: indentPadding }}
                    className="flex-row items-center justify-between py-3 active:opacity-60"
                  >
                    <View className="flex-1 flex-row items-center gap-2">
                      {/* Interactive Collapse/Expand Chevron or Branch guide */}
                      {hasChildren ? (
                        <Pressable
                          onPress={(e) => {
                            e.stopPropagation?.()
                            toggleExpand(folder.id)
                          }}
                          hitSlop={10}
                          className="size-6 items-center justify-center rounded active:bg-accent"
                        >
                          {isCollapsed ? (
                            <ChevronRight size={16} color={colors.mutedForeground} />
                          ) : (
                            <ChevronDown size={16} color={colors.mutedForeground} />
                          )}
                        </Pressable>
                      ) : depth > 0 ? (
                        <View className="size-6 items-center justify-center">
                          <CornerDownRight
                            size={14}
                            color={colors.mutedForeground}
                            strokeWidth={2}
                          />
                        </View>
                      ) : (
                        <View className="w-1" />
                      )}

                      <View className="size-8 items-center justify-center rounded-lg bg-accent">
                        <FolderIcon
                          name={folder.icon || DEFAULT_FOLDER_ICON}
                          size={18}
                          color={colors.primary}
                          fill={colors.primary}
                        />
                      </View>
                      <Text
                        numberOfLines={1}
                        className={`flex-1 text-[15px] font-medium ${
                          isSelected ? "font-semibold text-primary" : "text-foreground"
                        }`}
                      >
                        {folder.name}
                      </Text>
                    </View>
                    {isSelected && <Check size={18} color={colors.primary} strokeWidth={2.5} />}
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
