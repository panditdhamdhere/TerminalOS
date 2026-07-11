use std::path::PathBuf;

use clap::{Parser, Subcommand};
use terminalos_config::ConfigLoader;
use terminalos_git::GitRepository;
use terminalos_indexer::ProjectIndexer;
use terminalos_search::{SearchEngine, SearchQuery};
use terminalos_workspace::{WorkspaceManager, WorkspaceStore};
use tokio::runtime::Runtime;

/// TerminalOS command-line interface.
#[derive(Debug, Parser)]
#[command(name = "tos", version, about = "TerminalOS CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Show git repository status
    Status {
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    /// Index a project for search
    Index {
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
        #[arg(short, long, default_value = ".terminalos/index")]
        index: PathBuf,
    },
    /// Search indexed project files
    Search {
        query: String,
        #[arg(short, long, default_value = ".terminalos/index")]
        index: PathBuf,
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Open a workspace
    Open { path: PathBuf },
    /// List recent workspaces from session store
    Workspaces {
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Print configuration path
    Config,
}

fn main() -> terminalos_shared::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status { path } => {
            let repo = GitRepository::discover(&path)?;
            let status = repo.status()?;
            println!(
                "Branch: {}",
                status.branch.unwrap_or_else(|| "none".to_string())
            );
            println!("Staged: {}", status.staged);
            println!("Modified: {}", status.modified);
            println!("Untracked: {}", status.untracked);
            println!("Clean: {}", status.is_clean);
        }
        Commands::Index { path, index } => {
            let indexer = ProjectIndexer::new(&path, &index);
            let stats = indexer.index_all()?;
            println!(
                "Indexed {} files ({} bytes)",
                stats.files_indexed, stats.bytes_indexed
            );
        }
        Commands::Search {
            query,
            index,
            limit,
        } => {
            let engine = SearchEngine::open(&index)?;
            let hits = engine.search(&SearchQuery { text: query, limit })?;
            for hit in hits {
                println!(
                    "[{:.2}] {} — {}",
                    hit.score,
                    hit.path,
                    truncate(&hit.content, 80)
                );
            }
        }
        Commands::Open { path } => {
            let mut manager = WorkspaceManager::new();
            let id = manager.open(&path)?;
            if let Some(ws) = manager.get(id) {
                println!("Opened workspace: {} ({})", ws.name, ws.path.display());
            }
        }
        Commands::Workspaces { limit } => {
            let loader = ConfigLoader::default_paths();
            let store_path = loader
                .config_file_path()
                .parent()
                .unwrap_or(std::path::Path::new(".terminalos"))
                .join("workspace.db");
            let runtime = Runtime::new()
                .map_err(|e| terminalos_shared::Error::Ui(format!("tokio runtime: {e}")))?;
            let workspaces = runtime.block_on(async move {
                let store = WorkspaceStore::open(&store_path).await?;
                store.list_recent(limit).await
            })?;
            if workspaces.is_empty() {
                println!("No saved workspaces.");
            } else {
                for ws in workspaces {
                    let branch = ws.branch.unwrap_or_else(|| "none".to_string());
                    println!(
                        "{} — {} ({}) — {}",
                        ws.name,
                        ws.path,
                        branch,
                        ws.last_opened_at.format("%Y-%m-%d %H:%M")
                    );
                }
            }
        }
        Commands::Config => {
            let loader = ConfigLoader::default_paths();
            println!("{}", loader.config_file_path().display());
        }
    }

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}
