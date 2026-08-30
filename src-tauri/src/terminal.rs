use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use pty::{Command, PtySize};
use std::pin::Pin;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tauri::Emitter;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use tokio_util::io::MapperBytes;

/// Represents a terminal session
pub struct TerminalSession {
    /// The PTY master side
    master: pty::Master,
    /// Sender for input to the PTY
    input_tx: mpsc::Sender<String>,
    /// Receiver for output from the PTY
    output_rx: mpsc::Receiver<String>,
    /// Current working directory
    cwd: Option<String>,
    /// App handle for emitting events
    app_handle: tauri::AppHandle,
    /// Session ID
    id: String,
}

impl TerminalSession {
    /// Create a new terminal session with the specified command
    pub fn new(
        command: Command,
        cwd: Option<String>,
        app_handle: tauri::AppHandle,
        id: String,
    ) -> Result<Self, String> {
        let (master, slave) = pty::openpty().map_err(|e| e.to_string())?;

        // Spawn the command in the PTY
        let child = Command::new(command)
            .arg("-c") // For shell commands
            .spawn_slave(&slave)
            .map_err(|e| e.to_string())?;

        // We'll handle reading/writing in separate tasks
        // For simplicity, we'll use channels to communicate with the PTY
        let (input_tx, input_rx) = mpsc::channel();
        let (output_tx, output_rx) = mpsc::channel();

        // Clone the master for the reader task
        let mut master_reader = master.try_clone().map_err(|e| e.to_string())?;
        // Clone the master for the writer task
        let mut master_writer = master.try_clone().map_err(|e| e.to_string())?;
        // Clone the app handle for the output task
        let app_handle_clone = app_handle.clone();
        let session_id = id.clone();

        // Task to read from PTY and send to output channel
        tokio::spawn(async move {
            let mut buffer = vec![0; 1024];
            loop {
                match master_reader.read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        if let Ok(data) = String::from_utf8(buffer[..n].to_vec()) {
                            // Send to output channel
                            let _ = output_tx.send(data.clone());
                            // Emit event to frontend
                            let _ = app_handle_clone.emit(
                                "terminal-output",
                                (
                                    session_id.clone(),
                                    data, // The output data
                                ),
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading from PTY: {}", e);
                        break;
                    }
                }
            }
        });

        // Task to read from input channel and write to PTY
        tokio::spawn(async move {
            while let Ok(data) = input_rx.recv() {
                if let Err(e) = master_writer.write_all(data.as_bytes()) {
                    eprintln!("Error writing to PTY: {}", e);
                    break;
                }
            }
        });

        Ok(TerminalSession {
            master,
            input_tx,
            output_rx,
            cwd,
            app_handle,
            id,
        })
    }

    /// Send input to the terminal
    pub fn send_input(&self, input: String) -> Result<(), String> {
        self.input_tx
            .send(input)
            .map_err(|e| e.to_string())
    }

    /// Receive output from the terminal (non-blocking)
    pub fn try_recv_output(&self) -> Option<String> {
        self.output_rx.try_recv().ok()
    }

    /// Resize the terminal
    pub fn resize(&self, rows: u16, cols: u16) -> Result<(), String> {
        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        self.master
            .resize(size)
            .map_err(|e| e.to_string())
    }

    /// Get the session ID
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// Manager for terminal sessions
pub struct TerminalManager {
    sessions: Arc<Mutex<std::collections::HashMap<String, TerminalSession>>>,
    next_id: Arc<Mutex<u64>>,
    app_handle: tauri::AppHandle,
}

impl TerminalManager {
    /// Create a new terminal manager
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(std::collections::HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
            app_handle,
        }
    }

    /// Create a new terminal session
    pub fn create_session(
        &self,
        command: Command,
        cwd: Option<String>,
    ) -> Result<String, String> {
        let mut next_id = self.next_id.lock().map_err(|e| e.to_string())?;
        let id = format!("term-{}", *next_id);
        *next_id += 1;

        let session = TerminalSession::new(command, cwd, self.app_handle.clone(), id.clone())?;

        let mut sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        sessions.insert(id.clone(), session);

        Ok(id)
    }

    /// Get a terminal session by ID
    pub fn get_session(&self, id: &str) -> Option<TerminalSession> {
        let sessions = self.sessions.lock().ok()?;
        sessions.get(id).cloned()
    }

    /// Remove a terminal session
    pub fn remove_session(&self, id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        sessions.remove(id);
        Ok(())
    }

    /// List all session IDs
    pub fn list_sessions(&self) -> Vec<String> {
        let sessions = self.sessions.lock().ok().unwrap_or_default();
        sessions.keys().cloned().collect()
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use pty::Command;

    // Note: Unit tests for Tauri apps are complex because they require a runtime.
    // We'll skip unit tests for now and rely on manual testing.
}