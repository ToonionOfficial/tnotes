import {
  BottomSheetBackdrop,
  type BottomSheetBackdropProps,
  BottomSheetModal,
  BottomSheetTextInput,
  BottomSheetView,
} from "@gorhom/bottom-sheet"
import { forwardRef, useCallback, useImperativeHandle, useRef, useState } from "react"
import { Keyboard, Platform, Pressable, ScrollView, Text, View } from "react-native"
import { useCreateFolder } from "@/hooks/useFolders"
import { DEFAULT_FOLDER_ICON, FolderIcon, FOLDER_ICON_OPTIONS } from "./FolderIcon"

interface NewFolderSheetProps {
  onCreated: (folderId: string) => void
}

export interface NewFolderSheetRef {
  open: () => void
  close: () => void
}

export const NewFolderSheet = forwardRef<NewFolderSheetRef, NewFolderSheetProps>(
  function NewFolderSheet({ onCreated }, ref) {
    const bottomSheetModalRef = useRef<BottomSheetModal>(null)
    const [folderName, setFolderName] = useState("")
    const [selectedIcon, setSelectedIcon] = useState(DEFAULT_FOLDER_ICON)
    const createFolderMutation = useCreateFolder()

    const resetForm = useCallback(() => {
      setFolderName("")
      setSelectedIcon(DEFAULT_FOLDER_ICON)
    }, [])

    const close = useCallback(() => {
      Keyboard.dismiss()
      bottomSheetModalRef.current?.dismiss()
      resetForm()
    }, [resetForm])

    const open = useCallback(() => {
      resetForm()
      bottomSheetModalRef.current?.present()
    }, [resetForm])

    useImperativeHandle(ref, () => ({ open, close }), [open, close])

    const handleCreate = async () => {
      const trimmed = folderName.trim()
      if (!trimmed) return

      const created = await createFolderMutation.mutateAsync({
        name: trimmed,
        icon: selectedIcon,
      })

      close()
      onCreated(created.id)
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

    const handleDismiss = useCallback(() => {
      Keyboard.dismiss()
      resetForm()
    }, [resetForm])

    return (
      <BottomSheetModal
        ref={bottomSheetModalRef}
        enableDynamicSizing
        enablePanDownToClose
        keyboardBehavior="interactive"
        keyboardBlurBehavior="none"
        android_keyboardInputMode="adjustResize"
        onDismiss={handleDismiss}
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
        <BottomSheetView className="px-5 pt-1 pb-6">
          <Pressable onPress={Keyboard.dismiss} accessible={false}>
            <Text className="mb-4 text-[19px] font-bold text-foreground">New Folder</Text>

            <View className="mb-4 flex-row items-center overflow-hidden rounded-3xl bg-white/[0.07] px-4 py-3">
              <View className="mr-3 items-center justify-center">
                <FolderIcon name={selectedIcon} size={22} color="#CABEFF" fill="#CABEFF" />
              </View>
              <BottomSheetTextInput
                value={folderName}
                onChangeText={setFolderName}
                placeholder="Folder name"
                placeholderTextColor="#6E6B77"
                cursorColor="#CABEFF"
                autoFocus
                returnKeyType="done"
                onSubmitEditing={handleCreate}
                style={{
                  flex: 1,
                  fontSize: 17,
                  fontWeight: "500",
                  color: "#FFFFFF",
                }}
              />
            </View>
          </Pressable>

          <ScrollView
            horizontal
            showsHorizontalScrollIndicator={true}
            keyboardShouldPersistTaps="always"
            contentContainerStyle={{ gap: 8, paddingHorizontal: 4 }}
            className="mb-5"
          >
            {FOLDER_ICON_OPTIONS.map((iconName) => {
              const isSelected = selectedIcon === iconName
              return (
                <Pressable
                  key={iconName}
                  onPress={() => setSelectedIcon(iconName)}
                  className={`size-11 items-center justify-center rounded-2xl ${
                    isSelected ? "bg-primary/20" : "active:bg-white/10"
                  }`}
                >
                  <FolderIcon
                    name={iconName}
                    size={20}
                    color={isSelected ? "#CABEFF" : "#8E8C99"}
                    fill={isSelected ? "#CABEFF" : "#8E8C99"}
                  />
                </Pressable>
              )
            })}
          </ScrollView>

          <View className="flex-row items-center justify-end gap-3">
            <Pressable onPress={close} className="rounded-xl px-4 py-2.5 active:bg-white/6">
              <Text className="text-[15px] font-medium text-muted-foreground">Cancel</Text>
            </Pressable>
            <Pressable
              onPress={handleCreate}
              disabled={!folderName.trim()}
              className={`rounded-xl bg-primary px-5 py-2.5 active:opacity-80 ${
                !folderName.trim() ? "opacity-30" : ""
              }`}
            >
              <Text className="text-[15px] font-semibold text-[#141318]">Create</Text>
            </Pressable>
          </View>
        </BottomSheetView>
      </BottomSheetModal>
    )
  },
)
