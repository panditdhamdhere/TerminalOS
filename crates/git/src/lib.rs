//! Git repository inspection and operations.

pub mod blame;
pub mod conflict;
pub mod diff;
pub mod health;
pub mod repo;
pub mod staging;

pub use blame::{BlameEntry, blame_file, format_blame};
pub use conflict::{ConflictFile, ConflictRegion, find_conflicts, format_conflicts};
pub use diff::{
    FileDiff, commit_log_against_ref, diff_against_ref, diff_for_path, format_diffs, staged_diff,
    unstaged_diff,
};
pub use health::{HealthCheck, HealthReport, HealthStatus, assess, format_report};
pub use repo::{GitRepository, RepoStatus};
pub use staging::{ChangedFile, format_changed, list_changed, stage_command, unstage_command};
