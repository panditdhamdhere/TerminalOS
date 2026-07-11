use clap::Parser;
use terminalos_protocol::{DaemonRequest, DaemonResponse, Message, MessageKind};
use terminalos_workspace::WorkspaceManager;
use tracing::info;
use tracing_subscriber::EnvFilter;

/// TerminalOS background daemon for workspace and session management.
#[derive(Debug, Parser)]
#[command(name = "terminalos-daemon", version, about)]
struct Cli {
    /// Port for IPC (reserved for future Unix socket / TCP transport)
    #[arg(long, default_value = "9477")]
    port: u16,
}

#[tokio::main]
async fn main() -> terminalos_shared::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("terminalos=info".parse().expect("valid directive")),
        )
        .with_writer(std::io::stderr)
        .init();

    info!("TerminalOS daemon starting on port {}", cli.port);

    let mut workspaces = WorkspaceManager::new();

    if let Ok(cwd) = std::env::current_dir() {
        let _ = workspaces.open(&cwd);
        info!("Opened workspace: {}", cwd.display());
    }

    let health = Message::new(MessageKind::DaemonResponse(DaemonResponse::Ok {
        message: format!("daemon healthy, port {}", cli.port),
    }));

    if let MessageKind::DaemonResponse(DaemonResponse::Ok { message }) = health.kind {
        info!("{message}");
    }

    let _list = Message::new(MessageKind::DaemonRequest(DaemonRequest::ListWorkspaces));
    let count = workspaces.list().len();
    info!("Managing {count} workspace(s)");

    tokio::signal::ctrl_c()
        .await
        .map_err(|e| terminalos_shared::Error::Other(e.into()))?;

    info!("TerminalOS daemon shutting down");
    Ok(())
}
