import { Archive, Database } from "lucide-react-native"
import { memo } from "react"
import { SettingsRow } from "./SettingsRow"
import { SettingsSection } from "./SettingsSection"

export interface DataStorageSectionProps {
  storageSize?: string
  onPressExport?: () => void
}

export const DataStorageSection = memo(function DataStorageSection({
  storageSize = "SQLite (Local)",
  onPressExport,
}: DataStorageSectionProps) {
  return (
    <SettingsSection title="Storage">
      <SettingsRow
        icon={<Database size={20} color="#E6E1E9" />}
        title="Database"
        value={storageSize}
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
