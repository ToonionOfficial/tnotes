import * as SecureStore from "expo-secure-store"
import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react"
import { useColorScheme } from "react-native"
import { Uniwind } from "uniwind"
import {
  type AccentColorId,
  DEFAULT_ACCENT_COLOR_ID,
  getAccentColorPreset,
} from "@/constants/accentColors"

export type ThemePreference = "system" | "dark" | "light"
export type ResolvedTheme = "dark" | "light"

const THEME_STORAGE_KEY = "tnotes_theme_preference"
const ACCENT_STORAGE_KEY = "tnotes_accent_color_preference"

export interface ThemeColors {
  background: string
  foreground: string
  card: string
  muted: string
  mutedForeground: string
  primary: string
  border: string
}

export const DARK_BASE_COLORS = {
  background: "#141318",
  foreground: "#E6E1E9",
  card: "#201F24",
  muted: "#201F24",
  mutedForeground: "#C6C2CD",
  border: "#302E36",
}

export const LIGHT_BASE_COLORS = {
  background: "#F8F7FA",
  foreground: "#1D1B20",
  card: "#FFFFFF",
  muted: "#F3EEF8",
  mutedForeground: "#79747E",
  border: "#E6E0E9",
}

interface AppThemeContextValue {
  preference: ThemePreference
  theme: ResolvedTheme
  isDarkMode: boolean
  accentColor: AccentColorId
  colors: ThemeColors
  setPreference: (pref: ThemePreference) => void
  setAccentColor: (accent: AccentColorId) => void
  toggleTheme: (isDark: boolean) => void
}

const AppThemeContext = createContext<AppThemeContextValue>({
  preference: "system",
  theme: "dark",
  isDarkMode: true,
  accentColor: DEFAULT_ACCENT_COLOR_ID,
  colors: {
    ...DARK_BASE_COLORS,
    primary: getAccentColorPreset(DEFAULT_ACCENT_COLOR_ID).dark,
  },
  setPreference: () => {},
  setAccentColor: () => {},
  toggleTheme: () => {},
})

function applyAccentToUniwind(accentId: AccentColorId) {
  const preset = getAccentColorPreset(accentId)
  Uniwind.updateCSSVariables("light", {
    "--primary": preset.light,
    "--ring": preset.light,
  })
  Uniwind.updateCSSVariables("dark", {
    "--primary": preset.dark,
    "--ring": preset.dark,
  })
}

export function AppThemeProvider({ children }: { children: ReactNode }) {
  const systemColorScheme = useColorScheme()
  const [preference, setPreferenceState] = useState<ThemePreference>("system")
  const [accentColor, setAccentColorState] = useState<AccentColorId>(DEFAULT_ACCENT_COLOR_ID)

  useEffect(() => {
    async function loadSavedPreferences() {
      try {
        const [savedTheme, savedAccent] = await Promise.all([
          SecureStore.getItemAsync(THEME_STORAGE_KEY),
          SecureStore.getItemAsync(ACCENT_STORAGE_KEY),
        ])

        if (savedTheme === "light" || savedTheme === "dark" || savedTheme === "system") {
          setPreferenceState(savedTheme)
          Uniwind.setTheme(savedTheme)
        } else {
          setPreferenceState("system")
          Uniwind.setTheme("system")
        }

        if (savedAccent) {
          const validAccent = savedAccent as AccentColorId
          setAccentColorState(validAccent)
          applyAccentToUniwind(validAccent)
        }
      } catch {
        setPreferenceState("system")
        Uniwind.setTheme("system")
      }
    }
    void loadSavedPreferences()
  }, [])

  const setPreference = useCallback((newPref: ThemePreference) => {
    setPreferenceState(newPref)
    Uniwind.setTheme(newPref)
    void SecureStore.setItemAsync(THEME_STORAGE_KEY, newPref).catch(() => {})
  }, [])

  const setAccentColor = useCallback((newAccent: AccentColorId) => {
    setAccentColorState(newAccent)
    applyAccentToUniwind(newAccent)
    void SecureStore.setItemAsync(ACCENT_STORAGE_KEY, newAccent).catch(() => {})
  }, [])

  const toggleTheme = useCallback(
    (isDark: boolean) => {
      const nextPref: ThemePreference = isDark ? "dark" : "light"
      setPreference(nextPref)
    },
    [setPreference],
  )

  const resolvedTheme: ResolvedTheme = useMemo(() => {
    if (preference === "dark") return "dark"
    if (preference === "light") return "light"
    return systemColorScheme === "dark" ? "dark" : "light"
  }, [preference, systemColorScheme])

  const isDarkMode = resolvedTheme === "dark"
  const currentAccentPreset = useMemo(() => getAccentColorPreset(accentColor), [accentColor])

  const colors: ThemeColors = useMemo(() => {
    const primary = isDarkMode ? currentAccentPreset.dark : currentAccentPreset.light
    const base = isDarkMode ? DARK_BASE_COLORS : LIGHT_BASE_COLORS
    return {
      ...base,
      primary,
    }
  }, [isDarkMode, currentAccentPreset])

  return (
    <AppThemeContext.Provider
      value={{
        preference,
        theme: resolvedTheme,
        isDarkMode,
        accentColor,
        colors,
        setPreference,
        setAccentColor,
        toggleTheme,
      }}
    >
      {children}
    </AppThemeContext.Provider>
  )
}

export function useAppTheme() {
  return useContext(AppThemeContext)
}
