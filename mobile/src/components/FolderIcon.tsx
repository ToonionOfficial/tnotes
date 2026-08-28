import {
  Bookmark,
  Briefcase,
  Code,
  FileText,
  Folder,
  GraduationCap,
  Heart,
  Home,
  Lightbulb,
  Music,
  Palette,
  Rocket,
  ShoppingCart,
  Star,
  Target,
  Wallet,
  Zap,
  type LucideProps,
} from "lucide-react-native"
import { memo, type ComponentType } from "react"

const ICON_MAP: Record<string, ComponentType<LucideProps>> = {
  folder: Folder,
  briefcase: Briefcase,
  lightbulb: Lightbulb,
  "file-text": FileText,
  rocket: Rocket,
  target: Target,
  "graduation-cap": GraduationCap,
  palette: Palette,
  home: Home,
  wallet: Wallet,
  star: Star,
  heart: Heart,
  bookmark: Bookmark,
  code: Code,
  music: Music,
  zap: Zap,
  "shopping-cart": ShoppingCart,
}

export const FOLDER_ICON_OPTIONS = Object.keys(ICON_MAP)

export const DEFAULT_FOLDER_ICON = "folder"

interface FolderIconProps {
  name: string
  size?: number
  color?: string
  fill?: string
}

export const FolderIcon = memo(function FolderIcon({
  name,
  size = 18,
  color = "#CABEFF",
  fill,
}: FolderIconProps) {
  const IconComponent = ICON_MAP[name] || ICON_MAP[DEFAULT_FOLDER_ICON]
  return <IconComponent size={size} color={color} fill={fill} />
})
