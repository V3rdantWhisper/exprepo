use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

// ---------- CVE / Exp metadata (stored as meta.toml) ----------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CveMeta {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub references: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpMeta {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub status: String,
    /// Name of the target environment this exp is reproduced against.
    #[serde(default)]
    pub target_env: String,
    #[serde(default)]
    pub notes: String,
}

// ---------- Aggregate views returned to the frontend ----------

#[derive(Debug, Clone, Serialize)]
pub struct Exp {
    /// Directory name under `exps/`.
    pub id: String,
    pub cve_id: String,
    pub meta: ExpMeta,
    /// Relative (to repo root) paths of writeup markdown files under `wp/`.
    pub wps: Vec<String>,
    /// Relative (to repo root) paths of source files under `src/`.
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Cve {
    /// Directory name, e.g. `CVE-2023-1234`.
    pub id: String,
    pub meta: CveMeta,
    pub exps: Vec<Exp>,
}

// ---------- Path helpers ----------

pub fn cves_dir(repo: &Path) -> PathBuf {
    repo.join("cves")
}

pub fn cve_dir(repo: &Path, cve_id: &str) -> PathBuf {
    cves_dir(repo).join(cve_id)
}

pub fn exp_dir(repo: &Path, cve_id: &str, exp_id: &str) -> PathBuf {
    cve_dir(repo, cve_id).join("exps").join(exp_id)
}

pub fn guides_dir(repo: &Path) -> PathBuf {
    repo.join("guides")
}

/// Resolve a repo-relative path and guarantee it stays inside the repo.
pub fn resolve_in_repo(repo: &Path, rel: &str) -> AppResult<PathBuf> {
    let candidate = repo.join(rel);
    // Reject traversal explicitly; canonicalize the existing prefix when possible.
    if rel.split(['/', '\\']).any(|c| c == "..") {
        return Err(AppError::msg("path traversal is not allowed"));
    }
    Ok(candidate)
}

// ---------- meta.toml read/write ----------

fn read_toml<T: serde::de::DeserializeOwned + Default>(path: &Path) -> AppResult<T> {
    if !path.exists() {
        return Ok(T::default());
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&raw)?)
}

fn write_toml<T: Serialize>(path: &Path, value: &T) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, toml::to_string_pretty(value)?)?;
    Ok(())
}

pub fn read_cve_meta(repo: &Path, cve_id: &str) -> AppResult<CveMeta> {
    read_toml(&cve_dir(repo, cve_id).join("meta.toml"))
}

pub fn write_cve_meta(repo: &Path, cve_id: &str, meta: &CveMeta) -> AppResult<()> {
    write_toml(&cve_dir(repo, cve_id).join("meta.toml"), meta)
}

pub fn read_exp_meta(repo: &Path, cve_id: &str, exp_id: &str) -> AppResult<ExpMeta> {
    read_toml(&exp_dir(repo, cve_id, exp_id).join("meta.toml"))
}

pub fn write_exp_meta(repo: &Path, cve_id: &str, exp_id: &str, meta: &ExpMeta) -> AppResult<()> {
    write_toml(&exp_dir(repo, cve_id, exp_id).join("meta.toml"), meta)
}

// ---------- Scanning ----------

/// List the relative file paths (to repo root) directly contained in `dir`,
/// recursively, skipping `meta.toml`. Returns sorted paths.
fn list_files_rel(repo: &Path, dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    if !dir.exists() {
        return out;
    }
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        if entry.file_name() == "meta.toml" {
            continue;
        }
        if let Ok(rel) = entry.path().strip_prefix(repo) {
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    out.sort();
    out
}

fn subdir_names(dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                names.push(entry.file_name().to_string_lossy().to_string());
            }
        }
    }
    names.sort();
    names
}

pub fn scan_exps(repo: &Path, cve_id: &str) -> AppResult<Vec<Exp>> {
    let exps_root = cve_dir(repo, cve_id).join("exps");
    let mut exps = Vec::new();
    for exp_id in subdir_names(&exps_root) {
        let base = exp_dir(repo, cve_id, &exp_id);
        let wps = list_files_rel(repo, &base.join("wp"))
            .into_iter()
            .filter(|p| p.ends_with(".md"))
            .collect();
        let sources = list_files_rel(repo, &base.join("src"));
        exps.push(Exp {
            meta: read_exp_meta(repo, cve_id, &exp_id)?,
            id: exp_id,
            cve_id: cve_id.to_string(),
            wps,
            sources,
        });
    }
    Ok(exps)
}

pub fn scan_cves(repo: &Path) -> AppResult<Vec<Cve>> {
    let root = cves_dir(repo);
    let mut cves = Vec::new();
    for cve_id in subdir_names(&root) {
        cves.push(Cve {
            meta: read_cve_meta(repo, &cve_id)?,
            exps: scan_exps(repo, &cve_id)?,
            id: cve_id,
        });
    }
    Ok(cves)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> PathBuf {
        let p = std::env::temp_dir().join(format!("exprepo-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn scan_finds_cve_exp_wp_and_sources() {
        let repo = tmp();
        let base = exp_dir(&repo, "CVE-2023-1", "poc");
        std::fs::create_dir_all(base.join("wp")).unwrap();
        std::fs::create_dir_all(base.join("src")).unwrap();
        std::fs::write(base.join("wp/writeup.md"), "# hi").unwrap();
        std::fs::write(base.join("src/exploit.c"), "int main(){}").unwrap();
        write_cve_meta(
            &repo,
            "CVE-2023-1",
            &CveMeta { title: "t".into(), ..Default::default() },
        )
        .unwrap();

        let cves = scan_cves(&repo).unwrap();
        assert_eq!(cves.len(), 1);
        assert_eq!(cves[0].id, "CVE-2023-1");
        assert_eq!(cves[0].meta.title, "t");
        assert_eq!(cves[0].exps.len(), 1);
        let exp = &cves[0].exps[0];
        assert_eq!(exp.id, "poc");
        assert!(exp.wps.iter().any(|w| w.ends_with("wp/writeup.md")));
        assert!(exp.sources.iter().any(|s| s.ends_with("src/exploit.c")));

        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn resolve_rejects_traversal() {
        let repo = tmp();
        assert!(resolve_in_repo(&repo, "../etc/passwd").is_err());
        assert!(resolve_in_repo(&repo, "cves/x/meta.toml").is_ok());
        std::fs::remove_dir_all(&repo).ok();
    }
}
