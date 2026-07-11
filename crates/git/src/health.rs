use terminalos_shared::Result;

use crate::GitRepository;
use crate::conflict::find_conflicts;

/// Health check severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Pass,
    Warn,
    Fail,
}

/// A single repository health check result.
#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub message: String,
}

/// Full repository health assessment.
#[derive(Debug, Clone)]
pub struct HealthReport {
    pub checks: Vec<HealthCheck>,
}

impl HealthReport {
    #[must_use]
    pub fn has_failures(&self) -> bool {
        self.checks.iter().any(|c| c.status == HealthStatus::Fail)
    }

    #[must_use]
    pub fn has_warnings(&self) -> bool {
        self.checks.iter().any(|c| c.status == HealthStatus::Warn)
    }
}

/// Runs repository health checks.
pub fn assess(repo: &GitRepository) -> Result<HealthReport> {
    let status = repo.status()?;
    let conflicts = find_conflicts(repo)?;
    let mut checks = Vec::new();

    checks.push(if status.is_clean {
        HealthCheck {
            name: "Working tree".to_string(),
            status: HealthStatus::Pass,
            message: "Clean working tree.".to_string(),
        }
    } else {
        HealthCheck {
            name: "Working tree".to_string(),
            status: HealthStatus::Warn,
            message: format!(
                "Uncommitted changes: {} staged, {} modified, {} untracked",
                status.staged, status.modified, status.untracked
            ),
        }
    });

    checks.push(if conflicts.is_empty() {
        HealthCheck {
            name: "Merge conflicts".to_string(),
            status: HealthStatus::Pass,
            message: "No conflict markers found.".to_string(),
        }
    } else {
        HealthCheck {
            name: "Merge conflicts".to_string(),
            status: HealthStatus::Fail,
            message: format!("{} file(s) with conflict markers", conflicts.len()),
        }
    });

    checks.push(if !repo.has_upstream() {
        HealthCheck {
            name: "Remote sync".to_string(),
            status: HealthStatus::Warn,
            message: "No upstream branch configured.".to_string(),
        }
    } else {
        match (status.ahead, status.behind) {
            (0, 0) => HealthCheck {
                name: "Remote sync".to_string(),
                status: HealthStatus::Pass,
                message: "Up to date with remote.".to_string(),
            },
            (ahead, 0) if ahead > 0 => HealthCheck {
                name: "Remote sync".to_string(),
                status: HealthStatus::Warn,
                message: format!("{ahead} commit(s) ahead of remote — push when ready"),
            },
            (0, behind) if behind > 0 => HealthCheck {
                name: "Remote sync".to_string(),
                status: HealthStatus::Warn,
                message: format!("{behind} commit(s) behind remote — pull recommended"),
            },
            (ahead, behind) => HealthCheck {
                name: "Remote sync".to_string(),
                status: HealthStatus::Warn,
                message: format!("Diverged: {ahead} ahead, {behind} behind remote"),
            },
        }
    });

    checks.push(if status.untracked > 20 {
        HealthCheck {
            name: "Untracked files".to_string(),
            status: HealthStatus::Warn,
            message: format!(
                "{} untracked files — consider .gitignore review",
                status.untracked
            ),
        }
    } else {
        HealthCheck {
            name: "Untracked files".to_string(),
            status: HealthStatus::Pass,
            message: format!("{} untracked file(s)", status.untracked),
        }
    });

    checks.push(HealthCheck {
        name: "Branch".to_string(),
        status: HealthStatus::Pass,
        message: status
            .branch
            .clone()
            .map(|b| format!("On branch `{b}`"))
            .unwrap_or_else(|| "Detached or unborn HEAD".to_string()),
    });

    Ok(HealthReport { checks })
}

#[must_use]
pub fn format_report(report: &HealthReport) -> String {
    let mut out = String::from("## Repository Health\n\n");
    for check in &report.checks {
        let icon = match check.status {
            HealthStatus::Pass => "✓",
            HealthStatus::Warn => "⚠",
            HealthStatus::Fail => "✗",
        };
        out.push_str(&format!("{icon} **{}** — {}\n", check.name, check.message));
    }
    out
}
