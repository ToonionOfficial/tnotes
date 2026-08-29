import {
  BottomSheetBackdrop,
  type BottomSheetBackdropProps,
  BottomSheetModal,
  BottomSheetView,
} from "@gorhom/bottom-sheet"
import * as Haptics from "expo-haptics"
import { FolderInput, Pin, Share2, Trash2 } from "lucide-react-native"
import { forwardRef, useCallback, useImperativeHandle, useRef, useState } from "react"
import { Platform, Pressable, Share, Text, View } from "react-native"
import type { SearchResult } from "@/db/queries"
import type { Note } from "@/db/schema"
import { formatNoteTime } from "@/utils/date"
import { stripHtml } from "@/utils/text"

interface NoteActionSheetProps {
  note?: Note | SearchResult | null
  isTrash?: boolean
  onTogglePin?: (note: Note | SearchResult) => void
  onMoveToFolder?: (note: Note | SearchResult) => void
  onTrash?: (note: Note | SearchResult) => void
  onRestore?: (note: Note | SearchResult) => void
  onDeletePermanently?: (note: Note | SearchResult) => void
}

export interface NoteActionSheetRef {
  open: (note?: Note | SearchResult) => void
  close: () => void
}

export const NoteActionSheet = forwardRef<NoteActionSheetRef, NoteActionSheetProps>(
  function NoteActionSheet(
    {
      note: propNote,
      isTrash = false,
      onTogglePin,
      onMoveToFolder,
      onTrash,
      onRestore,
      onDeletePermanently,
    },
    ref,
  ) {
    const bottomSheetModalRef = useRef<BottomSheetModal>(null)
    const [internalNote, setInternalNote] = useState<Note | SearchResult | null>(propNote ?? null)

    const activeNote = propNote ?? internalNote

    const open = useCallback((noteToOpen?: Note | SearchResult) => {
      if (noteToOpen) {
        setInternalNote(noteToOpen)
      }
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

    const previewText = activeNote
      ? "snippet" in activeNote && activeNote.snippet
        ? stripHtml(String(activeNote.snippet))
        : stripHtml(activeNote.body)
      : ""

    const handleShare = async () => {
      if (!activeNote) return
      close()
      void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
      try {
        const shareContent = activeNote.title
          ? `${activeNote.title}\n\n${previewText}`
          : previewText
        await Share.share({
          title: activeNote.title || "Note",
          message: shareContent,
        })
      } catch {}
    }

    return (
      <BottomSheetModal
        ref={bottomSheetModalRef}
        enableDynamicSizing
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
        {activeNote ? (
          <BottomSheetView className="px-5 pb-8 pt-2">
            {/* Note Preview Header */}
            <View className="mb-3 px-1">
              <View className="flex-row items-center gap-2">
                {!isTrash && activeNote.pinned && <Pin size={12} color="#CABEFF" fill="#CABEFF" />}
                <Text
                  numberOfLines={1}
                  className="flex-1 text-[17px] font-semibold text-foreground"
                >
                  {activeNote.title || "Untitled Note"}
                </Text>
              </View>
              <View className="mt-1 flex-row items-center gap-1.5">
                <Text className="text-[12px] text-muted-foreground/60">
                  {formatNoteTime(activeNote.updatedAt)}
                </Text>
                {previewText.length > 0 && (
                  <>
                    <Text className="text-[12px] text-muted-foreground/40">·</Text>
                    <Text numberOfLines={1} className="flex-1 text-[12px] text-muted-foreground/80">
                      {previewText}
                    </Text>
                  </>
                )}
              </View>
            </View>

            {/* Action Items List */}
            <View className="overflow-hidden">
              {!isTrash && onTogglePin && (
                <Pressable
                  onPress={() => {
                    close()
                    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
                    onTogglePin(activeNote)
                  }}
                  className="flex-row items-center justify-between py-3.5 active:opacity-60"
                >
                  <View className="flex-row items-center gap-3">
                    <Pin size={19} color="#CABEFF" fill={activeNote.pinned ? "#CABEFF" : "none"} />
                    <Text className="text-[15px] font-medium text-white">
                      {activeNote.pinned ? "Unpin Note" : "Pin Note"}
                    </Text>
                  </View>
                </Pressable>
              )}

              {!isTrash && onTogglePin && <View className="ml-8 h-[0.5px] bg-white/10" />}

              {!isTrash && onMoveToFolder && (
                <Pressable
                  onPress={() => {
                    close()
                    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
                    onMoveToFolder(activeNote)
                  }}
                  className="flex-row items-center justify-between py-3.5 active:opacity-60"
                >
                  <View className="flex-row items-center gap-3">
                    <FolderInput size={19} color="#E6E1E9" />
                    <Text className="text-[15px] font-medium text-white">Move to Folder...</Text>
                  </View>
                </Pressable>
              )}

              {!isTrash && onMoveToFolder && <View className="ml-8 h-[0.5px] bg-white/10" />}

              <Pressable
                onPress={handleShare}
                className="flex-row items-center justify-between py-3.5 active:opacity-60"
              >
                <View className="flex-row items-center gap-3">
                  <Share2 size={19} color="#E6E1E9" />
                  <Text className="text-[15px] font-medium text-white">Share Note</Text>
                </View>
              </Pressable>

              <View className="ml-8 h-[0.5px] bg-white/10" />

              {isTrash ? (
                <>
                  {onRestore && (
                    <Pressable
                      onPress={() => {
                        close()
                        void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
                        onRestore(activeNote)
                      }}
                      className="flex-row items-center justify-between py-3.5 active:opacity-60"
                    >
                      <View className="flex-row items-center gap-3">
                        <Text className="text-[15px] font-medium text-[#3F8CFF]">Restore Note</Text>
                      </View>
                    </Pressable>
                  )}

                  {onDeletePermanently && (
                    <Pressable
                      onPress={() => {
                        close()
                        void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
                        onDeletePermanently(activeNote)
                      }}
                      className="flex-row items-center justify-between py-3.5 active:opacity-60"
                    >
                      <View className="flex-row items-center gap-3">
                        <Trash2 size={19} color="#FF6B6B" />
                        <Text className="text-[15px] font-medium text-[#FF6B6B]">
                          Delete Permanently
                        </Text>
                      </View>
                    </Pressable>
                  )}
                </>
              ) : (
                onTrash && (
                  <Pressable
                    onPress={() => {
                      close()
                      void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
                      onTrash(activeNote)
                    }}
                    className="flex-row items-center justify-between py-3.5 active:opacity-60"
                  >
                    <View className="flex-row items-center gap-3">
                      <Trash2 size={19} color="#FF6B6B" />
                      <Text className="text-[15px] font-medium text-[#FF6B6B]">Move to Trash</Text>
                    </View>
                  </Pressable>
                )
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
