use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMPORARY_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct PythonWorker {
    process: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl PythonWorker {
    pub fn spawn(
        python_executable: &Path,
        script_path: &Path,
        arguments: &[&str],
    ) -> Result<Self, String> {
        let mut process = Command::new(python_executable)
            .arg(script_path)
            .args(arguments)
            .env("PYTHONIOENCODING", "utf-8")
            .env("PYTHONWARNINGS", "ignore")
            .env("HF_HUB_DISABLE_SYMLINKS_WARNING", "1")
            .env("HF_HUB_DISABLE_PROGRESS_BARS", "1")
            .env("TRANSFORMERS_VERBOSITY", "error")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|error| format!("failed to start {}: {error}", python_executable.display()))?;
        let Some(input) = process.stdin.take() else {
            stop_process(&mut process);
            return Err("worker stdin is unavailable".to_owned());
        };
        let Some(output) = process.stdout.take() else {
            stop_process(&mut process);
            return Err("worker stdout is unavailable".to_owned());
        };
        Ok(Self {
            process,
            input,
            output: BufReader::new(output),
        })
    }

    pub fn send_line(&mut self, line: &str) -> Result<(), String> {
        writeln!(self.input, "{line}")
            .map_err(|error| format!("failed to write to worker: {error}"))?;
        self.input
            .flush()
            .map_err(|error| format!("failed to flush worker input: {error}"))
    }

    pub fn read_line(&mut self) -> Result<Option<String>, String> {
        let mut line = String::new();
        let bytes_read = self
            .output
            .read_line(&mut line)
            .map_err(|error| format!("failed to read worker output: {error}"))?;
        if bytes_read == 0 {
            Ok(None)
        } else {
            Ok(Some(line))
        }
    }
}

impl Drop for PythonWorker {
    fn drop(&mut self) {
        stop_process(&mut self.process);
    }
}

fn stop_process(process: &mut Child) {
    let _ = process.kill();
    let _ = process.wait();
}

pub fn manifest_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}

pub struct TemporaryImage {
    path: PathBuf,
}

impl TemporaryImage {
    pub fn new() -> Self {
        let file_counter = TEMPORARY_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let file_name = format!(
            "frames_translator_{}_{}.png",
            std::process::id(),
            file_counter
        );
        Self {
            path: std::env::temp_dir().join(file_name),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryImage {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
