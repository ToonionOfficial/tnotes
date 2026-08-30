export type AccentColorId =
  | "lavender"
  | "sapphire"
  | "emerald"
  | "coral"
  | "rose"
  | "amber"
  | "monochrome"

export interface AccentColorPreset {
  id: AccentColorId
  label: string
  preview: string
  light: string
  dark: string
}

export const ACCENT_COLOR_PRESETS: AccentColorPreset[] = [
  {
    id: "lavender",
    label: "Lavender",
    preview: "#9B86EC",
    light: "#65558F",
    dark: "#CABEFF",
  },
  {
    id: "sapphire",
    label: "Sapphire",
    preview: "#3898EC",
    light: "#00629E",
    dark: "#92CCFF",
  },
  {
    id: "emerald",
    label: "Emerald",
    preview: "#2ECC71",
    light: "#006C4C",
    dark: "#76DAA9",
  },
  {
    id: "coral",
    label: "Coral",
    preview: "#FF7043",
    light: "#A83800",
    dark: "#FFB59A",
  },
  {
    id: "rose",
    label: "Rose",
    preview: "#F06292",
    light: "#991A54",
    dark: "#FFB0CE",
  },
  {
    id: "amber",
    label: "Amber",
    preview: "#FFB300",
    light: "#7B5800",
    dark: "#FFDF9E",
  },
  {
    id: "monochrome",
    label: "Monochrome",
    preview: "#8E8E93",
    light: "#3A3A3C",
    dark: "#E5E5EA",
  },
]

export const DEFAULT_ACCENT_COLOR_ID: AccentColorId = "lavender"

export function getAccentColorPreset(id: AccentColorId): AccentColorPreset {
  return ACCENT_COLOR_PRESETS.find((preset) => preset.id === id) ?? ACCENT_COLOR_PRESETS[0]
}
