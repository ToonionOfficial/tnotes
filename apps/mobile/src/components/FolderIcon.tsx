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
  type LucideProps,
  Music,
  Palette,
  Rocket,
  ShoppingCart,
  Star,
  Target,
  Wallet,
  Zap,
} from "lucide-react-native"
import { type ComponentType, memo } from "react"

export const ICON_MAP = {
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
} as const satisfies Record<string, ComponentType<LucideProps>>

export type FolderIconName = keyof typeof ICON_MAP

export const FOLDER_ICON_OPTIONS = Object.keys(ICON_MAP) as readonly FolderIconName[]

export const DEFAULT_FOLDER_ICON: FolderIconName = "folder"

export function isFolderIconName(name: string): name is FolderIconName {
  return name in ICON_MAP
}

interface FolderIconProps {
  name?: FolderIconName | string | null
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
  const safeName = name && isFolderIconName(name) ? name : DEFAULT_FOLDER_ICON
  const IconComponent = ICON_MAP[safeName]
  return <IconComponent size={size} color={color} fill={fill} />
})
