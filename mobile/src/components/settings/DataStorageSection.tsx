import { Archive, Database } from "lucide-react-native"
import { memo } from "react"
import type { DatabaseStats } from "@/db/queries"
import { SettingsRow } from "./SettingsRow"
import { SettingsSection } from "./SettingsSection"

export interface DataStorageSectionProps {
  stats?: DatabaseStats
  onPressExport?: () => void
}

export const DataStorageSection = memo(function DataStorageSection({
  stats,
  onPressExport,
}: DataStorageSectionProps) {
  const subtitle = stats ? `${stats.notesCount} notes • ${stats.foldersCount} folders` : undefined

  return (
    <SettingsSection title="Storage">
      <SettingsRow
        icon={<Database size={20} color="#E6E1E9" />}
        title="Local Database"
        subtitle={subtitle}
        value={stats?.formattedSize ?? "Calculating..."}
        showChevron={false}
        showDivider
      />
      <SettingsRow
        icon={<Archive size={20} color="#E6E1E9" />}
        title="Export Notes"
        value="Markdown / JSON"
        onPress={onPressExport}
      />
    </SettingsSection>
  )
})
