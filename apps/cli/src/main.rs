use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use terminalos_config::ConfigLoader;
use terminalos_git::GitRepository;
use terminalos_indexer::{ProjectIndexer, hybrid_config_from_embedding, semantic_db_for_index};
use terminalos_search::{
    EmbeddingConfig, HybridSearchEngine, SearchEngine, SearchMode, SearchQuery,
};
use terminalos_workspace::{WorkspaceManager, WorkspaceStore};
use tokio::runtime::Runtime;

/// TerminalOS command-line interface.
#[derive(Debug, Parser)]
#[command(name = "tos", version, about = "TerminalOS CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum CliSearchMode {
    Hybrid,
    Keyword,
    Semantic,
}

impl From<CliSearchMode> for SearchMode {
    fn from(value: CliSearchMode) -> Self {
        match value {
            CliSearchMode::Hybrid => Self::Hybrid,
            CliSearchMode::Keyword => Self::Keyword,
            CliSearchMode::Semantic => Self::Semantic,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Show git repository status
    Status {
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    /// Index a project for keyword and semantic search
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
        #[arg(long, value_enum, default_value = "hybrid")]
        mode: CliSearchMode,
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
    let config = ConfigLoader::default_paths().load().unwrap_or_default();

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
            let embedding = EmbeddingConfig {
                base_url: config.search.embedding_base_url.clone(),
                model: config.search.embedding_model.clone(),
                api_key_env: config.search.embedding_api_key_env.clone(),
            };
            let indexer = ProjectIndexer::new(&path, &index).with_embedding_config(embedding);
            let stats = indexer.index_all()?;
            println!(
                "Indexed {} files ({} bytes, {} semantic chunks)",
                stats.files_indexed, stats.bytes_indexed, stats.semantic_chunks
            );
        }
        Commands::Search {
            query,
            index,
            limit,
            mode,
        } => {
            if mode == CliSearchMode::Keyword {
                let engine = SearchEngine::open(&index)?;
                let hits = engine.search(&SearchQuery { text: query, limit })?;
                print_hits(&hits);
            } else {
                let mut hybrid = hybrid_config_from_embedding(EmbeddingConfig {
                    base_url: config.search.embedding_base_url,
                    model: config.search.embedding_model,
                    api_key_env: config.search.embedding_api_key_env,
                });
                hybrid.mode = mode.into();
                hybrid.keyword_weight = config.search.keyword_weight;
                hybrid.semantic_weight = config.search.semantic_weight;

                let runtime = Runtime::new()
                    .map_err(|e| terminalos_shared::Error::Ui(format!("tokio runtime: {e}")))?;
                let engine = HybridSearchEngine::new(&index, semantic_db_for_index(&index), hybrid);
                let hits = runtime.block_on(engine.search(&SearchQuery { text: query, limit }))?;
                print_hits(&hits);
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

fn print_hits(hits: &[terminalos_search::SearchHit]) {
    for hit in hits {
        let location = match (hit.symbol.as_deref(), hit.start_line) {
            (Some(symbol), Some(line)) => format!("{symbol}:{line}"),
            (_, Some(line)) => format!("line {line}"),
            _ => String::new(),
        };
        let label = if location.is_empty() {
            hit.path.clone()
        } else {
            format!("{location} in {}", hit.path)
        };
        println!(
            "[{:.2}] {} — {}",
            hit.score,
            label,
            truncate(&hit.content, 80)
        );
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}
