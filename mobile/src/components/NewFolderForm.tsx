import {
  BottomSheetBackdrop,
  type BottomSheetBackdropProps,
  BottomSheetModal,
  BottomSheetTextInput,
  BottomSheetView,
} from "@gorhom/bottom-sheet"
import { zodResolver } from "@hookform/resolvers/zod"
import { forwardRef, useCallback, useImperativeHandle, useRef, useState } from "react"
import { Controller, useForm } from "react-hook-form"
import { Keyboard, Platform, Pressable, ScrollView, Text, View } from "react-native"
import { z } from "zod"
import type { Folder } from "@/db/schema"
import { useCreateFolder, useUpdateFolder } from "@/hooks/useFolders"
import {
  DEFAULT_FOLDER_ICON,
  FOLDER_ICON_OPTIONS,
  FolderIcon,
  type FolderIconName,
  isFolderIconName,
} from "./FolderIcon"

const folderSchema = z.object({
  name: z.string().trim().min(1, "Folder name is required").max(60, "Folder name is too long"),
  icon: z.custom<FolderIconName>((val) => typeof val === "string" && isFolderIconName(val), {
    message: "Invalid icon",
  }),
  parentId: z.string().nullable(),
})

export type FolderFormData = z.infer<typeof folderSchema>

interface NewFolderSheetProps {
  parentId?: string | null
  onCreated?: (folderId: string) => void
  onUpdated?: (folder: Folder) => void
}

export interface NewFolderSheetRef {
  open: (folderToEdit?: Folder | null, parentIdOverride?: string | null) => void
  close: () => void
}

export const NewFolderSheet = forwardRef<NewFolderSheetRef, NewFolderSheetProps>(
  function NewFolderSheet({ parentId: defaultParentId = null, onCreated, onUpdated }, ref) {
    const bottomSheetModalRef = useRef<BottomSheetModal>(null)
    const [editingFolder, setEditingFolder] = useState<Folder | null>(null)

    const createFolderMutation = useCreateFolder()
    const updateFolderMutation = useUpdateFolder()

    const {
      control,
      handleSubmit,
      reset,
      setValue,
      watch,
      formState: { isValid, isSubmitting },
    } = useForm<FolderFormData>({
      resolver: zodResolver(folderSchema),
      mode: "onChange",
      defaultValues: {
        name: "",
        icon: DEFAULT_FOLDER_ICON,
        parentId: defaultParentId ?? null,
      },
    })

    const selectedIcon = watch("icon")
    const folderName = watch("name")

    const close = useCallback(() => {
      Keyboard.dismiss()
      bottomSheetModalRef.current?.dismiss()
      setEditingFolder(null)
      reset({
        name: "",
        icon: DEFAULT_FOLDER_ICON,
        parentId: defaultParentId ?? null,
      })
    }, [defaultParentId, reset])

    const open = useCallback(
      (folderToEdit?: Folder | null, parentIdOverride?: string | null) => {
        if (folderToEdit) {
          setEditingFolder(folderToEdit)
          const validIcon: FolderIconName =
            folderToEdit.icon && isFolderIconName(folderToEdit.icon)
              ? folderToEdit.icon
              : DEFAULT_FOLDER_ICON
          reset({
            name: folderToEdit.name,
            icon: validIcon,
            parentId: folderToEdit.parentId ?? null,
          })
        } else {
          setEditingFolder(null)
          reset({
            name: "",
            icon: DEFAULT_FOLDER_ICON,
            parentId: parentIdOverride !== undefined ? parentIdOverride : (defaultParentId ?? null),
          })
        }
        bottomSheetModalRef.current?.present()
      },
      [defaultParentId, reset],
    )

    useImperativeHandle(ref, () => ({ open, close }), [open, close])

    const onSubmit = async (data: FolderFormData) => {
      const trimmed = data.name.trim()
      if (!trimmed) return

      if (editingFolder) {
        const updated = await updateFolderMutation.mutateAsync({
          id: editingFolder.id,
          input: {
            name: trimmed,
            icon: data.icon,
          },
        })
        close()
        onUpdated?.(updated)
      } else {
        const created = await createFolderMutation.mutateAsync({
          name: trimmed,
          icon: data.icon,
          parentId: data.parentId,
        })
        close()
        onCreated?.(created.id)
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

    const handleDismiss = useCallback(() => {
      Keyboard.dismiss()
      setEditingFolder(null)
      reset({
        name: "",
        icon: DEFAULT_FOLDER_ICON,
        parentId: defaultParentId ?? null,
      })
    }, [defaultParentId, reset])

    const isEditing = Boolean(editingFolder)

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
            <Text className="mb-4 text-[19px] font-bold text-foreground">
              {isEditing ? "Edit Folder" : "New Folder"}
            </Text>

            <View className="mb-4 flex-row items-center overflow-hidden rounded-3xl bg-white/[0.07] px-4 py-3">
              <View className="mr-3 items-center justify-center">
                <FolderIcon name={selectedIcon} size={22} color="#CABEFF" fill="#CABEFF" />
              </View>
              <Controller
                control={control}
                name="name"
                render={({ field: { onChange, onBlur, value } }) => (
                  <BottomSheetTextInput
                    value={value}
                    onChangeText={onChange}
                    onBlur={onBlur}
                    placeholder="Folder name"
                    placeholderTextColor="#6E6B77"
                    cursorColor="#CABEFF"
                    autoFocus
                    returnKeyType="done"
                    onSubmitEditing={handleSubmit(onSubmit)}
                    style={{
                      flex: 1,
                      fontSize: 17,
                      fontWeight: "500",
                      color: "#FFFFFF",
                    }}
                  />
                )}
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
                  onPress={() => setValue("icon", iconName, { shouldValidate: true })}
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
              onPress={handleSubmit(onSubmit)}
              disabled={!folderName?.trim() || !isValid || isSubmitting}
              className={`rounded-xl bg-primary px-5 py-2.5 active:opacity-80 ${
                !folderName?.trim() || !isValid || isSubmitting ? "opacity-30" : ""
              }`}
            >
              <Text className="text-[15px] font-semibold text-[#141318]">
                {isEditing ? "Save" : "Create"}
              </Text>
            </Pressable>
          </View>
        </BottomSheetView>
      </BottomSheetModal>
    )
  },
)
