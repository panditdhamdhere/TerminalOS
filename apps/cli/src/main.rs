use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use terminalos_config::{
    ConfigLoader, builtin_preset_names, load_env_files, provider_statuses, set_default_provider,
};
use terminalos_git::GitRepository;
use terminalos_indexer::{ProjectIndexer, hybrid_config_from_embedding, semantic_db_for_index};
use terminalos_plugin::{PluginInstaller, PluginManager, PluginMarketplace};
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
    /// Configuration management
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommands>,
    },
    /// Manage plugins
    Plugins {
        #[command(subcommand)]
        command: PluginCommands,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommands {
    /// Print the active configuration file path
    Path,
    /// Show resolved configuration summary
    Show,
    /// List built-in theme presets
    Themes,
    /// Manage configuration profiles
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
    /// Manage AI providers
    Provider {
        #[command(subcommand)]
        command: ProviderCommands,
    },
}

#[derive(Debug, Subcommand)]
enum ProviderCommands {
    /// List configured AI providers
    List,
    /// Set the default enabled provider
    Use { name: String },
}

#[derive(Debug, Subcommand)]
enum ProfileCommands {
    /// List available profiles
    List,
    /// Set the active profile in config.toml
    Use { name: String },
}

#[derive(Debug, Subcommand)]
enum PluginCommands {
    /// List installed plugins
    List,
    /// Show marketplace catalog
    Marketplace,
    /// Install a plugin from the marketplace
    Install { name: String },
    /// Run a plugin command
    Run {
        plugin: String,
        command: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

fn main() -> terminalos_shared::Result<()> {
    let cli = Cli::parse();
    let loader = ConfigLoader::default_paths();
    load_env_files(&loader);
    let config = loader.load().unwrap_or_default();

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
        Commands::Config { command } => {
            let loader = ConfigLoader::default_paths();
            match command {
                None | Some(ConfigCommands::Path) => {
                    println!("{}", loader.config_file_path().display());
                }
                Some(ConfigCommands::Show) => {
                    let config = loader.load().unwrap_or_default();
                    println!("Config: {}", loader.config_file_path().display());
                    println!(
                        "Profile: {}",
                        config.active_profile.as_deref().unwrap_or("none")
                    );
                    println!("Theme: {:?}", config.ui.theme);
                    if let Some(preset) = &config.ui.theme_preset {
                        println!("Theme preset: {preset}");
                    }
                    println!("Providers: {}", config.providers.len());
                    for provider in &config.providers {
                        let status = if provider.enabled { "on" } else { "off" };
                        println!("  - {} ({status})", provider.name);
                    }
                    println!("Workspace autosave: {}s", config.workspace.autosave_secs);
                }
                Some(ConfigCommands::Themes) => {
                    println!("Built-in theme presets:");
                    for name in builtin_preset_names() {
                        println!("  - {name}");
                    }
                }
                Some(ConfigCommands::Profile { command }) => match command {
                    ProfileCommands::List => {
                        loader.ensure_default()?;
                        for name in loader.list_profiles()? {
                            println!("{name}");
                        }
                    }
                    ProfileCommands::Use { name } => {
                        let config = loader.set_active_profile(&name)?;
                        println!("Active profile set to: {name}");
                        println!("Sidebar: {}", config.ui.show_sidebar);
                        println!("Chat: {}", config.ui.show_chat);
                        println!("Logs: {}", config.ui.show_logs);
                    }
                },
                Some(ConfigCommands::Provider { command }) => match command {
                    ProviderCommands::List => {
                        for status in provider_statuses(&config) {
                            let default = if status.is_default { " [default]" } else { "" };
                            let ready = if status.ready { "ready" } else { "not ready" };
                            let enabled = if status.enabled { "on" } else { "off" };
                            println!(
                                "{} ({enabled}, {ready}){} — {}",
                                status.name, default, status.model
                            );
                        }
                    }
                    ProviderCommands::Use { name } => {
                        let config = set_default_provider(&loader, &name)?;
                        println!("Default provider set to: {name}");
                        if let Some(provider) = config.providers.iter().find(|p| p.name == name) {
                            println!("Model: {}", provider.model);
                        }
                    }
                },
            }
        }
        Commands::Plugins { command } => match command {
            PluginCommands::List => {
                let mut manager = PluginManager::new(PluginManager::default_dir());
                manager.load_all()?;
                if manager.plugins().is_empty() {
                    println!("No plugins installed.");
                } else {
                    for plugin in manager.plugins() {
                        let status = if plugin.dynamic.is_some() {
                            "loaded"
                        } else {
                            "manifest only"
                        };
                        println!(
                            "{} v{} — {} [{status}]",
                            plugin.info().name,
                            plugin.info().version,
                            plugin.info().description
                        );
                        for cmd in plugin.commands() {
                            println!("  - {}: {}", cmd.name, cmd.description);
                        }
                    }
                }
            }
            PluginCommands::Marketplace => {
                let market = PluginMarketplace::bundled();
                for entry in market.entries() {
                    println!("{} v{} — {}", entry.name, entry.version, entry.description);
                    println!("  author: {}", entry.author);
                    println!("  commands: {}", entry.commands.join(", "));
                }
            }
            PluginCommands::Install { name } => {
                let market = PluginMarketplace::bundled();
                let entry = market.find(&name).ok_or_else(|| {
                    terminalos_shared::Error::Plugin(format!("unknown plugin: {name}"))
                })?;
                let installer = PluginInstaller::new(PluginManager::default_dir());
                let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
                let install_dir = installer.install_from_source(entry, &repo_root)?;
                println!("Installed {} to {}", entry.name, install_dir.display());
            }
            PluginCommands::Run {
                plugin,
                command,
                args,
            } => {
                let mut manager = PluginManager::new(PluginManager::default_dir());
                manager.load_all()?;
                let output = manager.execute(&plugin, &command, &args)?;
                println!("{output}");
            }
        },
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
