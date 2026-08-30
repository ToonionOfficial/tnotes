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

export type ThemePreference = "system" | "dark" | "light"
export type ResolvedTheme = "dark" | "light"

const THEME_STORAGE_KEY = "tnotes_theme_preference"

export interface ThemeColors {
  background: string
  foreground: string
  card: string
  muted: string
  mutedForeground: string
  primary: string
  border: string
}

export const DARK_COLORS: ThemeColors = {
  background: "#141318",
  foreground: "#E6E1E9",
  card: "#201F24",
  muted: "#201F24",
  mutedForeground: "#C6C2CD",
  primary: "#CABEFF",
  border: "#302E36",
}

export const LIGHT_COLORS: ThemeColors = {
  background: "#F8F7FA",
  foreground: "#1D1B20",
  card: "#FFFFFF",
  muted: "#F3EEF8",
  mutedForeground: "#79747E",
  primary: "#65558F",
  border: "#E6E0E9",
}

interface AppThemeContextValue {
  preference: ThemePreference
  theme: ResolvedTheme
  isDarkMode: boolean
  colors: ThemeColors
  setPreference: (pref: ThemePreference) => void
  toggleTheme: (isDark: boolean) => void
}

const AppThemeContext = createContext<AppThemeContextValue>({
  preference: "system",
  theme: "dark",
  isDarkMode: true,
  colors: DARK_COLORS,
  setPreference: () => {},
  toggleTheme: () => {},
})

export function AppThemeProvider({ children }: { children: ReactNode }) {
  const systemColorScheme = useColorScheme()
  const [preference, setPreferenceState] = useState<ThemePreference>("system")

  useEffect(() => {
    async function loadSavedTheme() {
      try {
        const saved = await SecureStore.getItemAsync(THEME_STORAGE_KEY)
        if (saved === "light" || saved === "dark" || saved === "system") {
          setPreferenceState(saved)
          Uniwind.setTheme(saved)
        } else {
          setPreferenceState("system")
          Uniwind.setTheme("system")
        }
      } catch {
        setPreferenceState("system")
        Uniwind.setTheme("system")
      }
    }
    void loadSavedTheme()
  }, [])

  const setPreference = useCallback((newPref: ThemePreference) => {
    setPreferenceState(newPref)
    Uniwind.setTheme(newPref)
    void SecureStore.setItemAsync(THEME_STORAGE_KEY, newPref).catch(() => {})
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
  const colors = isDarkMode ? DARK_COLORS : LIGHT_COLORS

  return (
    <AppThemeContext.Provider
      value={{
        preference,
        theme: resolvedTheme,
        isDarkMode,
        colors,
        setPreference,
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
