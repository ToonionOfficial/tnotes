/// Format an epoch-ms timestamp as a relative time string
/// ("Just now", "5m ago", "3h ago", "Yesterday", "4d ago").
pub fn format_relative_time(epoch_ms: i64) -> String {
    let now = current_time_ms();
    let delta_secs = (now - epoch_ms).max(0) / 1000;

    if delta_secs < 60 {
        return "Just now".to_string();
    }
    let minutes = delta_secs / 60;
    if minutes < 60 {
        return format!("{minutes}m ago");
    }
    let hours = minutes / 60;
    if hours < 24 {
        return format!("{hours}h ago");
    }
    if hours < 48 {
        return "Yesterday".to_string();
    }
    let days = hours / 24;
    format!("{days}d ago")
}

fn current_time_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Extract a plain-text snippet from a note body, truncated to `max_chars`.
/// Truncates at a word boundary when possible and appends "…" on truncation.
pub fn snippet_from_body(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    let mut out: String = trimmed.chars().take(max_chars).collect();
    // Prefer a word boundary: cut back to the last space if one exists
    // in the back third of the truncated string.
    let cutoff = max_chars.saturating_sub(max_chars / 3);
    if let Some(last_space) = out.rfind(' ')
        && out.len() > cutoff
    {
        out.truncate(last_space);
    }
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_time_buckets() {
        let now = current_time_ms();
        assert_eq!(format_relative_time(now), "Just now");
        assert_eq!(format_relative_time(now - 30_000), "Just now");
        assert_eq!(format_relative_time(now - 5 * 60_000), "5m ago");
        assert_eq!(format_relative_time(now - 2 * 3_600_000), "2h ago");
        assert_eq!(format_relative_time(now - 30 * 3_600_000), "Yesterday");
        assert_eq!(format_relative_time(now - 4 * 86_400_000), "4d ago");
    }

    #[test]
    fn snippet_truncates_with_ellipsis() {
        assert_eq!(snippet_from_body("hello", 120), "hello");
        let long: String = "word ".repeat(50);
        let out = snippet_from_body(&long, 120);
        assert!(out.chars().count() <= 121, "unexpected len: {}", out.len());
        assert!(out.ends_with('…'));
    }
}
