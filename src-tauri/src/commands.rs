use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, State};

use crate::config::AppConfig;
use crate::env_runner::{self, EnvRunner, Environment};
use crate::error::{AppError, AppResult};
use crate::model::{self, Cve, CveMeta, ExpMeta};
use crate::{repo, secrets};

/// Shared, managed application state.
pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub config_path: PathBuf,
    pub envs_path: PathBuf,
    pub runner: EnvRunner,
}

impl AppState {
    fn repo_path(&self) -> PathBuf {
        self.config.lock().unwrap().repo_path.clone()
    }

    fn save_config(&self) -> AppResult<()> {
        self.config.lock().unwrap().save(&self.config_path)
    }
}

fn rel_to_repo(repo: &std::path::Path, path: &std::path::Path) -> String {
    path.strip_prefix(repo)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

// ---------------- config / repo ----------------

#[tauri::command]
pub fn get_config(state: State<AppState>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_repo_path(state: State<AppState>, path: String) -> AppResult<AppConfig> {
    {
        let mut cfg = state.config.lock().unwrap();
        cfg.repo_path = PathBuf::from(path);
    }
    state.save_config()?;
    Ok(state.config.lock().unwrap().clone())
}

#[tauri::command]
pub fn repo_init(state: State<AppState>) -> AppResult<()> {
    repo::open_or_init(&state.repo_path())?;
    Ok(())
}

#[tauri::command]
pub fn repo_clone(state: State<AppState>, url: String) -> AppResult<()> {
    let path = state.repo_path();
    if path.join(".git").exists() {
        return Err(AppError::msg("a repository already exists at the repo path"));
    }
    let token = secrets::get_token()?;
    repo::clone(&url, &path, token)?;
    {
        let mut cfg = state.config.lock().unwrap();
        cfg.remote_url = Some(url);
    }
    state.save_config()?;
    Ok(())
}

#[tauri::command]
pub fn repo_status(state: State<AppState>) -> AppResult<repo::RepoStatus> {
    let r = repo::open(&state.repo_path())?;
    repo::status(&r)
}

#[tauri::command]
pub fn repo_commit(state: State<AppState>, message: String) -> AppResult<String> {
    let r = repo::open(&state.repo_path())?;
    let (name, email) = {
        let cfg = state.config.lock().unwrap();
        (cfg.git_name.clone(), cfg.git_email.clone())
    };
    repo::commit_all(&r, &message, &name, &email)
}

#[tauri::command]
pub fn repo_push(state: State<AppState>) -> AppResult<()> {
    let r = repo::open(&state.repo_path())?;
    let branch = state.config.lock().unwrap().default_branch.clone();
    let token = secrets::get_token()?;
    repo::push(&r, &branch, token)
}

#[tauri::command]
pub fn repo_pull(state: State<AppState>) -> AppResult<()> {
    let r = repo::open(&state.repo_path())?;
    let branch = state.config.lock().unwrap().default_branch.clone();
    let token = secrets::get_token()?;
    repo::pull(&r, &branch, token)
}

#[tauri::command]
pub fn repo_get_remote(state: State<AppState>) -> AppResult<Option<String>> {
    let r = repo::open(&state.repo_path())?;
    Ok(repo::get_remote_url(&r))
}

#[tauri::command]
pub fn repo_set_remote(state: State<AppState>, url: String) -> AppResult<()> {
    let r = repo::open(&state.repo_path())?;
    repo::set_remote_url(&r, &url)?;
    {
        let mut cfg = state.config.lock().unwrap();
        cfg.remote_url = Some(url);
    }
    state.save_config()
}

// ---------------- CVE / exp / wp ----------------

#[tauri::command]
pub fn cve_list(state: State<AppState>) -> AppResult<Vec<Cve>> {
    model::scan_cves(&state.repo_path())
}

#[tauri::command]
pub fn cve_get(state: State<AppState>, cve_id: String) -> AppResult<Cve> {
    let repo = state.repo_path();
    if !model::cve_dir(&repo, &cve_id).exists() {
        return Err(AppError::msg("CVE not found"));
    }
    Ok(Cve {
        meta: model::read_cve_meta(&repo, &cve_id)?,
        exps: model::scan_exps(&repo, &cve_id)?,
        id: cve_id,
    })
}

#[tauri::command]
pub fn cve_create(state: State<AppState>, cve_id: String, meta: CveMeta) -> AppResult<()> {
    let repo = state.repo_path();
    let dir = model::cve_dir(&repo, &cve_id);
    if dir.exists() {
        return Err(AppError::msg("CVE already exists"));
    }
    std::fs::create_dir_all(dir.join("exps"))?;
    model::write_cve_meta(&repo, &cve_id, &meta)
}

#[tauri::command]
pub fn cve_update_meta(state: State<AppState>, cve_id: String, meta: CveMeta) -> AppResult<()> {
    model::write_cve_meta(&state.repo_path(), &cve_id, &meta)
}

#[tauri::command]
pub fn exp_create(
    state: State<AppState>,
    cve_id: String,
    exp_id: String,
    meta: ExpMeta,
) -> AppResult<()> {
    let repo = state.repo_path();
    let base = model::exp_dir(&repo, &cve_id, &exp_id);
    if base.exists() {
        return Err(AppError::msg("exp already exists"));
    }
    std::fs::create_dir_all(base.join("src"))?;
    std::fs::create_dir_all(base.join("wp"))?;
    model::write_exp_meta(&repo, &cve_id, &exp_id, &meta)
}

#[tauri::command]
pub fn exp_update_meta(
    state: State<AppState>,
    cve_id: String,
    exp_id: String,
    meta: ExpMeta,
) -> AppResult<()> {
    model::write_exp_meta(&state.repo_path(), &cve_id, &exp_id, &meta)
}

/// Create a new (empty) writeup markdown file under an exp's `wp/` dir.
/// Returns the repo-relative path of the new file.
#[tauri::command]
pub fn wp_create(
    state: State<AppState>,
    cve_id: String,
    exp_id: String,
    filename: String,
) -> AppResult<String> {
    let repo = state.repo_path();
    let mut name = filename.trim().to_string();
    if name.is_empty() {
        return Err(AppError::msg("filename is required"));
    }
    if !name.ends_with(".md") {
        name.push_str(".md");
    }
    let dir = model::exp_dir(&repo, &cve_id, &exp_id).join("wp");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(&name);
    if path.exists() {
        return Err(AppError::msg("a writeup with that name already exists"));
    }
    let title = name.trim_end_matches(".md");
    std::fs::write(&path, format!("# {title}\n\n"))?;
    Ok(rel_to_repo(&repo, &path))
}

#[tauri::command]
pub fn file_read(state: State<AppState>, path: String) -> AppResult<String> {
    let repo = state.repo_path();
    let full = model::resolve_in_repo(&repo, &path)?;
    Ok(std::fs::read_to_string(full)?)
}

#[tauri::command]
pub fn file_write(state: State<AppState>, path: String, content: String) -> AppResult<()> {
    let repo = state.repo_path();
    let full = model::resolve_in_repo(&repo, &path)?;
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(full, content)?;
    Ok(())
}

// ---------------- environments ----------------

#[tauri::command]
pub fn env_list(state: State<AppState>) -> AppResult<Vec<Environment>> {
    env_runner::load_envs(&state.envs_path)
}

#[tauri::command]
pub fn env_running(state: State<AppState>) -> Vec<String> {
    state.runner.running_ids()
}

/// Create or update an environment. A new environment (empty `id`) is assigned
/// a fresh uuid and creation timestamp. Returns the stored record.
#[tauri::command]
pub fn env_save(state: State<AppState>, mut env: Environment) -> AppResult<Environment> {
    let mut envs = env_runner::load_envs(&state.envs_path)?;
    if env.id.is_empty() {
        env.id = uuid::Uuid::new_v4().to_string();
        env.created_at = chrono::Utc::now().to_rfc3339();
        envs.push(env.clone());
    } else if let Some(slot) = envs.iter_mut().find(|e| e.id == env.id) {
        *slot = env.clone();
    } else {
        envs.push(env.clone());
    }
    env_runner::save_envs(&state.envs_path, &envs)?;
    Ok(env)
}

#[tauri::command]
pub fn env_delete(state: State<AppState>, id: String) -> AppResult<()> {
    if state.runner.is_running(&id) {
        return Err(AppError::msg("stop the environment before deleting it"));
    }
    let mut envs = env_runner::load_envs(&state.envs_path)?;
    envs.retain(|e| e.id != id);
    env_runner::save_envs(&state.envs_path, &envs)
}

#[tauri::command]
pub fn env_launch(app: AppHandle, state: State<AppState>, id: String) -> AppResult<()> {
    let envs = env_runner::load_envs(&state.envs_path)?;
    let env = envs
        .into_iter()
        .find(|e| e.id == id)
        .ok_or_else(|| AppError::msg("environment not found"))?;
    state.runner.launch(&app, &env)
}

#[tauri::command]
pub fn env_stop(state: State<AppState>, id: String) -> AppResult<()> {
    state.runner.stop(&id)
}

/// Export an environment's build guide markdown to the repo's `guides/` dir so
/// it can be committed and synced. Returns the repo-relative path.
#[tauri::command]
pub fn env_export_guide(state: State<AppState>, id: String) -> AppResult<String> {
    let repo = state.repo_path();
    let envs = env_runner::load_envs(&state.envs_path)?;
    let env = envs
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| AppError::msg("environment not found"))?;
    let content = env
        .build_guide
        .clone()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| AppError::msg("environment has no build guide content"))?;
    let safe_name: String = env
        .name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect();
    let dir = model::guides_dir(&repo);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{safe_name}.md"));
    std::fs::write(&path, content)?;
    Ok(rel_to_repo(&repo, &path))
}

// ---------------- secrets ----------------

#[tauri::command]
pub fn secret_set_token(token: String) -> AppResult<()> {
    secrets::set_token(&token)
}

#[tauri::command]
pub fn secret_has_token() -> bool {
    secrets::has_token()
}

#[tauri::command]
pub fn secret_delete_token() -> AppResult<()> {
    secrets::delete_token()
}
