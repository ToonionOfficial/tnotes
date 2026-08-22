//! Version-wins conflict resolution.
//!
//! Resolution order:
//! 1. Higher `version` wins
//! 2. Tie: higher `updated_at` wins
//! 3. Tie: lexicographically higher `device_id` wins
