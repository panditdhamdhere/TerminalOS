use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use portable_pty::{CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
use terminalos_shared::{Error, PaneId, Result};
use tracing::{debug, warn};

/// Output event streamed from a PTY session.
#[derive(Debug, Clone)]
pub struct PtyOutput {
    pub pane_id: PaneId,
    pub data: Vec<u8>,
}

/// Handle to a running PTY shell process.
pub struct PtySession {
    pane_id: PaneId,
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    _child: Box<dyn portable_pty::Child + Send + Sync>,
}

impl PtySession {
    pub fn spawn(
        pane_id: PaneId,
        cwd: &str,
        rows: u16,
        cols: u16,
        output_tx: Sender<PtyOutput>,
        env: &HashMap<String, String>,
    ) -> Result<Self> {
        let pty_system = NativePtySystem::default();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| Error::Terminal(format!("open pty: {e}")))?;

        let shell = default_shell();
        let mut cmd = CommandBuilder::new(&shell);
        cmd.cwd(cwd);
        for (key, value) in env {
            cmd.env(key, value);
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| Error::Terminal(format!("spawn shell: {e}")))?;

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| Error::Terminal(format!("clone reader: {e}")))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| Error::Terminal(format!("take writer: {e}")))?;

        let reader_pane = pane_id;
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let data = buf[..n].to_vec();
                        if output_tx
                            .send(PtyOutput {
                                pane_id: reader_pane,
                                data,
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(e) => {
                        warn!(pane = %reader_pane.as_uuid(), error = %e, "pty read error");
                        break;
                    }
                }
            }
            debug!(pane = %reader_pane.as_uuid(), "pty reader exited");
        });

        Ok(Self {
            pane_id,
            master: pair.master,
            writer,
            _child: child,
        })
    }

    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.writer
            .write_all(data)
            .map_err(|e| Error::Terminal(format!("pty write: {e}")))?;
        self.writer
            .flush()
            .map_err(|e| Error::Terminal(format!("pty flush: {e}")))?;
        Ok(())
    }

    pub fn resize(&mut self, rows: u16, cols: u16) -> Result<()> {
        self.master
            .resize(PtySize {
                rows: rows.max(4),
                cols: cols.max(20),
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| Error::Terminal(format!("pty resize: {e}")))
    }

    #[must_use]
    pub fn pane_id(&self) -> PaneId {
        self.pane_id
    }
}

fn default_shell() -> String {
    #[cfg(windows)]
    {
        std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
    }
    #[cfg(not(windows))]
    {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
    }
}

/// Creates a channel pair for PTY output streaming.
#[must_use]
pub fn output_channel() -> (Sender<PtyOutput>, Receiver<PtyOutput>) {
    mpsc::channel()
}
