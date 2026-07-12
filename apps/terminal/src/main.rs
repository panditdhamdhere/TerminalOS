use std::path::PathBuf;

use clap::Parser;
use terminalos_config::ConfigLoader;
use terminalos_ui::{TerminalApp, TerminalAppOptions};
use tracing::info;
use tracing_subscriber::EnvFilter;

/// TerminalOS — The AI-native terminal for developers.
#[derive(Debug, Parser)]
#[command(name = "terminalos", version, about, long_about = None)]
struct Cli {
    /// Workspace directory to open
    #[arg(short, long)]
    workspace: Option<PathBuf>,

    /// Path to config directory
    #[arg(long)]
    config_dir: Option<PathBuf>,

    /// Configuration profile to load (default, minimal, coding, groq, or custom)
    #[arg(long)]
    profile: Option<String>,

    /// Skip the interactive first-run setup wizard
    #[arg(long)]
    skip_setup: bool,
}

fn main() -> terminalos_shared::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("terminalos=info".parse().expect("valid directive")),
        )
        .with_writer(std::io::stderr)
        .init();

    let loader = match cli.config_dir {
        Some(dir) => ConfigLoader::new(dir),
        None => ConfigLoader::default_paths(),
    };

    let config = if let Some(ref name) = cli.profile {
        info!("Loading profile: {name}");
        terminalos_config::load_env_files(&loader);
        let mut config = loader.load_with_profile(name)?;
        if terminalos_config::auto_configure_providers(&mut config) {
            loader.save(&config)?;
        }
        config
    } else {
        loader.ensure_ready(cli.skip_setup)?
    };
    info!("TerminalOS v{} starting", env!("CARGO_PKG_VERSION"));

    let mut app = TerminalApp::new(TerminalAppOptions {
        workspace_path: cli.workspace.or_else(|| std::env::current_dir().ok()),
        config,
    })?;

    app.run()
}
