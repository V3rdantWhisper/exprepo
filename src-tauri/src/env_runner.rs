use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::error::{AppError, AppResult};

/// A QEMU-based reproduction environment. Stored locally (never synced) in
/// `environments.json`; the optional `build_guide` markdown can be exported to
/// the exp repo's `guides/` directory on demand.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub arch: String,
    #[serde(default)]
    pub qemu_binary: String,
    #[serde(default)]
    pub kernel_image: Option<String>,
    #[serde(default)]
    pub rootfs: Option<String>,
    /// Kernel command line (`-append`).
    #[serde(default)]
    pub append: Option<String>,
    /// Raw extra QEMU args, shell-quoted.
    #[serde(default)]
    pub extra_args: String,
    #[serde(default)]
    pub working_dir: Option<String>,
    /// Build/setup guide markdown content (inline, editable in-app).
    #[serde(default)]
    pub build_guide: Option<String>,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Clone, Serialize)]
struct LogEvent {
    env_id: String,
    stream: String,
    line: String,
}

#[derive(Clone, Serialize)]
struct ExitEvent {
    env_id: String,
    code: Option<i32>,
}

type SharedChild = Arc<Mutex<Child>>;
type ProcMap = Arc<Mutex<HashMap<String, SharedChild>>>;

/// Tracks running QEMU processes keyed by environment id.
#[derive(Default)]
pub struct EnvRunner {
    procs: ProcMap,
}

fn default_qemu_binary(arch: &str) -> String {
    match arch {
        "" | "x86_64" | "amd64" => "qemu-system-x86_64".to_string(),
        other => format!("qemu-system-{other}"),
    }
}

fn build_args(env: &Environment) -> AppResult<Vec<String>> {
    let mut args = Vec::new();
    if let Some(k) = env.kernel_image.as_deref().filter(|s| !s.is_empty()) {
        args.push("-kernel".into());
        args.push(k.into());
    }
    if let Some(r) = env.rootfs.as_deref().filter(|s| !s.is_empty()) {
        args.push("-drive".into());
        args.push(format!("file={r},format=raw"));
    }
    if let Some(a) = env.append.as_deref().filter(|s| !s.is_empty()) {
        args.push("-append".into());
        args.push(a.into());
    }
    let extra = shell_words::split(&env.extra_args)
        .map_err(|e| AppError::msg(format!("invalid extra args: {e}")))?;
    args.extend(extra);
    Ok(args)
}

fn spawn_reader<R: Read + Send + 'static>(
    app: AppHandle,
    env_id: String,
    reader: R,
    stream: &'static str,
) {
    std::thread::spawn(move || {
        let buf = BufReader::new(reader);
        for line in buf.lines() {
            match line {
                Ok(l) => {
                    let _ = app.emit(
                        "env-log",
                        LogEvent {
                            env_id: env_id.clone(),
                            stream: stream.to_string(),
                            line: l,
                        },
                    );
                }
                Err(_) => break,
            }
        }
    });
}

impl EnvRunner {
    pub fn is_running(&self, id: &str) -> bool {
        self.procs.lock().unwrap().contains_key(id)
    }

    pub fn running_ids(&self) -> Vec<String> {
        self.procs.lock().unwrap().keys().cloned().collect()
    }

    pub fn launch(&self, app: &AppHandle, env: &Environment) -> AppResult<()> {
        if self.is_running(&env.id) {
            return Err(AppError::msg("environment is already running"));
        }
        let bin = if env.qemu_binary.is_empty() {
            default_qemu_binary(&env.arch)
        } else {
            env.qemu_binary.clone()
        };

        let mut cmd = Command::new(&bin);
        cmd.args(build_args(env)?);
        if let Some(wd) = env.working_dir.as_deref().filter(|s| !s.is_empty()) {
            cmd.current_dir(wd);
        }
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| AppError::msg(format!("failed to start {bin}: {e}")))?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        if let Some(out) = stdout {
            spawn_reader(app.clone(), env.id.clone(), out, "stdout");
        }
        if let Some(err) = stderr {
            spawn_reader(app.clone(), env.id.clone(), err, "stderr");
        }

        let shared: SharedChild = Arc::new(Mutex::new(child));
        self.procs
            .lock()
            .unwrap()
            .insert(env.id.clone(), shared.clone());

        // Monitor thread: poll for exit without holding the lock during the wait,
        // so `stop()` can still acquire it to kill the process.
        let app = app.clone();
        let id = env.id.clone();
        let procs_handle = self.procs.clone();
        std::thread::spawn(move || {
            let code = loop {
                {
                    let mut guard = shared.lock().unwrap();
                    match guard.try_wait() {
                        Ok(Some(status)) => break status.code(),
                        Ok(None) => {}
                        Err(_) => break None,
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(250));
            };
            procs_handle.lock().unwrap().remove(&id);
            let _ = app.emit("env-exit", ExitEvent { env_id: id, code });
        });

        Ok(())
    }

    pub fn stop(&self, id: &str) -> AppResult<()> {
        let shared = self
            .procs
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| AppError::msg("environment is not running"))?;
        let _ = shared.lock().unwrap().kill();
        Ok(())
    }
}

/// Load the environment list from `environments.json` (empty if missing).
pub fn load_envs(path: &Path) -> AppResult<Vec<Environment>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn save_envs(path: &Path, envs: &[Environment]) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(envs)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> Environment {
        Environment {
            id: String::new(),
            name: "x".into(),
            arch: "x86_64".into(),
            qemu_binary: String::new(),
            kernel_image: Some("/k/bzImage".into()),
            rootfs: Some("/r/rootfs.img".into()),
            append: Some("console=ttyS0".into()),
            extra_args: "-nographic -m 1G".into(),
            working_dir: None,
            build_guide: None,
            created_at: String::new(),
        }
    }

    #[test]
    fn build_args_composes_qemu_flags() {
        let a = build_args(&env()).unwrap();
        assert!(a.windows(2).any(|w| w == ["-kernel", "/k/bzImage"]));
        assert!(a.windows(2).any(|w| w == ["-append", "console=ttyS0"]));
        assert!(a.iter().any(|s| s.contains("file=/r/rootfs.img")));
        assert!(a.contains(&"-nographic".to_string()));
        assert!(a.contains(&"-m".to_string()));
        assert!(a.contains(&"1G".to_string()));
    }

    #[test]
    fn build_args_rejects_unbalanced_quotes() {
        let mut e = env();
        e.extra_args = "-foo \"unterminated".into();
        assert!(build_args(&e).is_err());
    }

    #[test]
    fn default_binary_follows_arch() {
        assert_eq!(default_qemu_binary("x86_64"), "qemu-system-x86_64");
        assert_eq!(default_qemu_binary("aarch64"), "qemu-system-aarch64");
        assert_eq!(default_qemu_binary(""), "qemu-system-x86_64");
    }
}
