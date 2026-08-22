//! Version-wins conflict resolution.
//!
//! Resolution order:
//! 1. Higher `version` wins
//! 2. Tie: higher `updated_at` wins
//! 3. Tie: lexicographically higher `device_id` wins

/// Metadata required to perform deterministic conflict resolution
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionMeta<'a> {
    pub version: u64,
    pub updated_at: i64,
    pub device_id: &'a str,
}

impl<'a> VersionMeta<'a> {
    pub fn new(version: u64, updated_at: i64, device_id: &'a str) -> Self {
        Self {
            version,
            updated_at,
            device_id,
        }
    }
}

/// Determines if an incoming remote change should overwrite the existing local record.
///
/// Returns `true` if remote wins (apply change), or `false` if local wins (ignore change).
pub fn should_apply_remote(local: &VersionMeta, remote: &VersionMeta) -> bool {
    // 1. Higher monotonic version number wins
    if remote.version != local.version {
        return remote.version > local.version;
    }

    // 2. Tiebreaker: later timestamp wins
    if remote.updated_at != local.updated_at {
        return remote.updated_at > local.updated_at;
    }

    // 3. Deterministic tiebreaker: lexicographically higher device_id wins
    remote.device_id > local.device_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_version_wins() {
        let local = VersionMeta::new(2, 1000, "dev_a");
        let remote = VersionMeta::new(3, 500, "dev_a");
        assert!(should_apply_remote(&local, &remote));

        let local_newer = VersionMeta::new(4, 1000, "dev_a");
        let remote_older = VersionMeta::new(3, 2000, "dev_a");
        assert!(!should_apply_remote(&local_newer, &remote_older));
    }

    #[test]
    fn test_timestamp_tiebreak() {
        let local = VersionMeta::new(2, 1000, "dev_a");
        let remote = VersionMeta::new(2, 2000, "dev_a");
        assert!(should_apply_remote(&local, &remote));

        let local = VersionMeta::new(2, 2000, "dev_a");
        let remote = VersionMeta::new(2, 1000, "dev_a");
        assert!(!should_apply_remote(&local, &remote));
    }

    #[test]
    fn test_device_id_tiebreak() {
        let local = VersionMeta::new(2, 1000, "dev_a");
        let remote = VersionMeta::new(2, 1000, "dev_b");
        assert!(should_apply_remote(&local, &remote));

        let local = VersionMeta::new(2, 1000, "dev_z");
        let remote = VersionMeta::new(2, 1000, "dev_a");
        assert!(!should_apply_remote(&local, &remote));
    }

    #[test]
    fn test_identical_meta() {
        let local = VersionMeta::new(2, 1000, "dev_a");
        let remote = VersionMeta::new(2, 1000, "dev_a");
        assert!(!should_apply_remote(&local, &remote));
    }
}
