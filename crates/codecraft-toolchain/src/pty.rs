use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use crate::runner::{ExecutionResult, ExecutionStreamChunk, StreamType};

/// Terminal stream controller untuk capture stdout/stderr non-blocking
pub struct TerminalStreamCapture {
    sender: Sender<ExecutionStreamChunk>,
    receiver: Receiver<ExecutionStreamChunk>,
}

impl Default for TerminalStreamCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalStreamCapture {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self { sender, receiver }
    }

    pub fn sender(&self) -> Sender<ExecutionStreamChunk> {
        self.sender.clone()
    }

    pub fn try_recv(&self) -> Option<ExecutionStreamChunk> {
        self.receiver.try_recv().ok()
    }

    pub fn emit_stdout(&self, text: impl Into<String>) {
        let _ = self.sender.send(ExecutionStreamChunk {
            stream_type: StreamType::Stdout,
            content: text.into(),
        });
    }

    pub fn emit_stderr(&self, text: impl Into<String>) {
        let _ = self.sender.send(ExecutionStreamChunk {
            stream_type: StreamType::Stderr,
            content: text.into(),
        });
    }

    pub fn emit_system(&self, text: impl Into<String>) {
        let _ = self.sender.send(ExecutionStreamChunk {
            stream_type: StreamType::SystemNotice,
            content: text.into(),
        });
    }
}

/// Helper eksekusi command dengan timeout dan capture output (cross-platform fallback)
pub fn execute_with_capture(
    program: &str,
    args: &[String],
    capture: &TerminalStreamCapture,
    timeout: Duration,
) -> Result<ExecutionResult, std::io::Error> {
    use std::process::{Command, Stdio};
    use std::io::{BufRead, BufReader};
    use std::time::Instant;

    let start_time = Instant::now();
    capture.emit_system(format!("$ {} {}\n", program, args.join(" ")));

    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();

    let tx_out = capture.sender();
    let tx_err = capture.sender();

    let out_handle = std::thread::spawn(move || {
        if let Some(pipe) = stdout_pipe {
            let reader = BufReader::new(pipe);
            for line in reader.lines().flatten() {
                let _ = tx_out.send(ExecutionStreamChunk {
                    stream_type: StreamType::Stdout,
                    content: format!("{}\n", line),
                });
            }
        }
    });

    let err_handle = std::thread::spawn(move || {
        if let Some(pipe) = stderr_pipe {
            let reader = BufReader::new(pipe);
            for line in reader.lines().flatten() {
                let _ = tx_err.send(ExecutionStreamChunk {
                    stream_type: StreamType::Stderr,
                    content: format!("{}\n", line),
                });
            }
        }
    });

    // Watchdog loop untuk timeout
    loop {
        match child.try_wait()? {
            Some(status) => {
                let _ = out_handle.join();
                let _ = err_handle.join();
                let exit_code = status.code().unwrap_or(-1);
                let duration_ms = start_time.elapsed().as_millis();

                capture.emit_system(format!(
                    "\n[Process exited with code {} in {}ms]\n",
                    exit_code, duration_ms
                ));

                return Ok(ExecutionResult {
                    exit_code,
                    success: status.success(),
                    total_duration_ms: duration_ms,
                    memory_peak_kb: None,
                });
            }
            None => {
                if start_time.elapsed() > timeout {
                    let _ = child.kill();
                    capture.emit_system("\n[Process terminated: Execution Timeout exceeded]\n");
                    return Ok(ExecutionResult {
                        exit_code: -9,
                        success: false,
                        total_duration_ms: start_time.elapsed().as_millis(),
                        memory_peak_kb: None,
                    });
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}
