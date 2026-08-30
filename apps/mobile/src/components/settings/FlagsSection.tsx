import { BarChart3, Trash2 } from "lucide-react-native"
import { useState } from "react"
import { Alert, Pressable, Text, View } from "react-native"
import { useAppTheme } from "@/hooks/useAppTheme"
import { useCreateBenchmarkNotes, useDeleteBenchmarkNotes } from "@/hooks/useNotes"
import { SettingsSection } from "./SettingsSection"

export const ENABLE_BENCHMARK = process.env.EXPO_PUBLIC_ENABLE_BENCHMARK === "true" || __DEV__

export function FlagsSection() {
  const { colors } = useAppTheme()
  const createBenchmark = useCreateBenchmarkNotes()
  const deleteBenchmark = useDeleteBenchmarkNotes()
  const [status, setStatus] = useState<string | null>(null)

  if (!ENABLE_BENCHMARK) return null

  const isRunning = createBenchmark.isPending || deleteBenchmark.isPending

  const addNotes = (count: number) => {
    Alert.alert(
      `Add ${count.toLocaleString()} benchmark notes?`,
      `This writes ${count.toLocaleString()} real notes across ${Math.ceil(count / 10).toLocaleString()} benchmark folders in SQLite and syncs them to your server.`,
      [
        { text: "Cancel", style: "cancel" },
        {
          text: "Add notes",
          onPress: () => {
            setStatus(`Adding ${count.toLocaleString()} notes…`)
            createBenchmark.mutate(count, {
              onSuccess: (result) => {
                setStatus(
                  `Added ${result.noteCount.toLocaleString()} notes in ${result.folderCount.toLocaleString()} folders · ${(result.elapsedMs / 1000).toFixed(1)}s`,
                )
              },
              onError: () => setStatus("Could not add benchmark notes."),
            })
          },
        },
      ],
    )
  }

  const removeNotes = () => {
    Alert.alert(
      "Delete benchmark notes?",
      "Only notes and folders created by this benchmark will be permanently deleted.",
      [
        { text: "Cancel", style: "cancel" },
        {
          text: "Delete notes",
          style: "destructive",
          onPress: () => {
            setStatus("Deleting benchmark notes…")
            deleteBenchmark.mutate(undefined, {
              onSuccess: (result) => {
                setStatus(
                  `Deleted ${result.noteCount.toLocaleString()} notes and ${result.folderCount.toLocaleString()} folders · ${(result.elapsedMs / 1000).toFixed(1)}s`,
                )
              },
              onError: () => setStatus("Could not delete benchmark notes."),
            })
          },
        },
      ],
    )
  }

  return (
    <SettingsSection title="Flags & Developer Tools">
      <View className="p-4">
        <View className="flex-row items-center gap-2">
          <BarChart3 size={18} color={colors.primary} />
          <Text className="text-[16px] font-semibold text-foreground">Performance Benchmark</Text>
        </View>
        <Text className="mt-1 text-[13px] leading-5 text-muted-foreground">
          Generates real notes in randomized folders with ~10 notes per folder and syncs them to the
          server.
        </Text>

        <View className="mt-4 flex-row gap-2">
          {[100, 500, 1000, 5000].map((count) => (
            <Pressable
              key={count}
              disabled={isRunning}
              onPress={() => addNotes(count)}
              className="flex-1 items-center rounded-xl bg-primary px-2 py-2.5 active:opacity-70 disabled:opacity-40"
            >
              <Text className="text-[12px] font-semibold text-primary-foreground">
                +{count >= 1000 ? `${count / 1000}k` : count}
              </Text>
            </Pressable>
          ))}
        </View>

        <Pressable
          disabled={isRunning}
          onPress={removeNotes}
          className="mt-2 flex-row items-center justify-center gap-2 rounded-xl bg-[#D94C5C]/20 py-2.5 active:opacity-70 disabled:opacity-40"
        >
          <Trash2 size={15} color="#FF6B6B" />
          <Text className="text-[13px] font-semibold text-[#FF6B6B]">Delete benchmark notes</Text>
        </Pressable>

        {status && <Text className="mt-3 text-[12px] text-muted-foreground">{status}</Text>}
      </View>
    </SettingsSection>
  )
}
