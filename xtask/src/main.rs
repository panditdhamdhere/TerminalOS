use std::path::PathBuf;
use std::process::{Command, ExitCode};

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "xtask", about = "TerminalOS developer automation tasks")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run the full local CI pipeline (fmt, clippy, test, snapshots)
    Ci,
    /// Format all workspace crates
    Fmt,
    /// Check formatting without writing changes
    FmtCheck,
    /// Run clippy with warnings denied
    Clippy,
    /// Run workspace tests
    Test,
    /// Run workspace benchmarks
    Bench,
    /// Run snapshot tests (use --update to refresh snapshots)
    Snapshot {
        /// Update snapshot files instead of comparing
        #[arg(long)]
        update: bool,
    },
    /// Build the mdBook documentation site
    Docs,
    /// Install git commit hooks
    Hooks,
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Commands::Ci => run_ci(),
        Commands::Fmt => run_fmt(false),
        Commands::FmtCheck => run_fmt(true),
        Commands::Clippy => run_clippy(),
        Commands::Test => run_test(),
        Commands::Bench => run_bench(),
        Commands::Snapshot { update } => run_snapshot(update),
        Commands::Docs => run_docs(),
        Commands::Hooks => run_hooks(),
    }
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn cargo() -> Command {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(project_root());
    cmd
}

fn run_command(mut cmd: Command) -> ExitCode {
    let status = cmd.status().expect("failed to spawn command");
    if status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(status.code().unwrap_or(1) as u8)
    }
}

fn run_fmt(check: bool) -> ExitCode {
    let mut cmd = cargo();
    cmd.arg("fmt").arg("--all");
    if check {
        cmd.args(["--", "--check"]);
    }
    run_command(cmd)
}

fn run_clippy() -> ExitCode {
    let mut cmd = cargo();
    cmd.args([
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ]);
    run_command(cmd)
}

fn run_test() -> ExitCode {
    let mut cmd = cargo();
    cmd.args(["test", "--workspace"]);
    run_command(cmd)
}

fn run_bench() -> ExitCode {
    let mut cmd = cargo();
    cmd.args(["bench", "--workspace"]);
    run_command(cmd)
}

fn run_snapshot(update: bool) -> ExitCode {
    let mut cmd = cargo();
    cmd.args(["test", "--workspace", "snapshot"]);
    if update {
        cmd.env("INSTA_UPDATE", "1");
    }
    run_command(cmd)
}

fn run_docs() -> ExitCode {
    let status = Command::new("mdbook")
        .current_dir(project_root())
        .arg("build")
        .status();

    match status {
        Ok(result) if result.success() => {
            println!("Documentation built at docs/book/");
            ExitCode::SUCCESS
        }
        Ok(_) => ExitCode::FAILURE,
        Err(err) => {
            eprintln!("mdbook failed: {err}");
            eprintln!("Install with: cargo install mdbook");
            ExitCode::FAILURE
        }
    }
}

fn run_hooks() -> ExitCode {
    let root = project_root();
    let hook_src = root.join("scripts/hooks/commit-msg");
    let hook_dst = root.join(".git/hooks/commit-msg");

    if !hook_src.exists() {
        eprintln!("Hook script not found: {}", hook_src.display());
        return ExitCode::FAILURE;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(&hook_src) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(&hook_src, perms);
        }
    }

    if let Err(err) = std::fs::copy(&hook_src, &hook_dst) {
        eprintln!("Failed to install hook: {err}");
        return ExitCode::FAILURE;
    }

    println!("Installed commit-msg hook at {}", hook_dst.display());
    ExitCode::SUCCESS
}

fn run_ci() -> ExitCode {
    for step in [run_fmt(true), run_clippy(), run_test(), run_snapshot(false)] {
        if step != ExitCode::SUCCESS {
            return step;
        }
    }
    ExitCode::SUCCESS
}
