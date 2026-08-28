import type { SearchResult } from "@/db/queries"
import type { Note } from "@/db/schema"

export interface NoteSection {
  title: string
  data: (Note | SearchResult)[]
}

/**
 * Formats timestamps for note list items (e.g. "10:45 AM" if today, otherwise "Jul 15").
 */
export function formatNoteTime(timestamp: number): string {
  const noteDate = new Date(timestamp)
  const now = new Date()

  const isToday =
    noteDate.getDate() === now.getDate() &&
    noteDate.getMonth() === now.getMonth() &&
    noteDate.getFullYear() === now.getFullYear()

  if (isToday) {
    return noteDate.toLocaleTimeString(undefined, {
      hour: "2-digit",
      minute: "2-digit",
    })
  }

  return noteDate.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
  })
}

/**
 * Groups notes into Apple Notes style chronological sections:
 * Pinned, Today, Yesterday, Previous 7 Days, Previous 30 Days, Month/Year
 */
export function groupNotesByDate(notes: (Note | SearchResult)[]): NoteSection[] {
  if (!notes || notes.length === 0) return []

  const now = new Date()
  const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime()
  const yesterdayStart = todayStart - 24 * 60 * 60 * 1000
  const prev7DaysStart = todayStart - 6 * 24 * 60 * 60 * 1000
  const prev30DaysStart = todayStart - 29 * 24 * 60 * 60 * 1000

  const pinned: (Note | SearchResult)[] = []
  const today: (Note | SearchResult)[] = []
  const yesterday: (Note | SearchResult)[] = []
  const prev7Days: (Note | SearchResult)[] = []
  const prev30Days: (Note | SearchResult)[] = []
  const monthGroups = new Map<string, (Note | SearchResult)[]>()

  for (const note of notes) {
    if (note.pinned) {
      pinned.push(note)
      continue
    }

    const t = note.updatedAt
    if (t >= todayStart) {
      today.push(note)
    } else if (t >= yesterdayStart) {
      yesterday.push(note)
    } else if (t >= prev7DaysStart) {
      prev7Days.push(note)
    } else if (t >= prev30DaysStart) {
      prev30Days.push(note)
    } else {
      const d = new Date(t)
      const label =
        d.getFullYear() === now.getFullYear()
          ? d.toLocaleDateString(undefined, { month: "long" })
          : d.toLocaleDateString(undefined, {
              month: "long",
              year: "numeric",
            })
      if (!monthGroups.has(label)) {
        monthGroups.set(label, [])
      }
      const group = monthGroups.get(label)
      if (group) {
        group.push(note)
      }
    }
  }

  const sections: NoteSection[] = []

  if (pinned.length > 0) sections.push({ title: "Pinned", data: pinned })
  if (today.length > 0) sections.push({ title: "Today", data: today })
  if (yesterday.length > 0) sections.push({ title: "Yesterday", data: yesterday })
  if (prev7Days.length > 0) sections.push({ title: "Previous 7 Days", data: prev7Days })
  if (prev30Days.length > 0) sections.push({ title: "Previous 30 Days", data: prev30Days })

  for (const [monthTitle, items] of monthGroups.entries()) {
    if (items.length > 0) {
      sections.push({ title: monthTitle, data: items })
    }
  }

  return sections
}
