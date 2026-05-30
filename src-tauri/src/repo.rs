use std::path::Path;

use git2::{
    build::RepoBuilder, Cred, FetchOptions, PushOptions, RemoteCallbacks, Repository, Signature,
};
use serde::Serialize;

use crate::error::{AppError, AppResult};

const REMOTE: &str = "origin";

/// A single entry from `git status`, simplified for the UI.
#[derive(Debug, Clone, Serialize)]
pub struct StatusEntry {
    pub path: String,
    /// One of: "new", "modified", "deleted", "renamed", "typechange", "conflicted".
    pub state: String,
    pub staged: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoStatus {
    pub branch: Option<String>,
    pub remote_url: Option<String>,
    pub entries: Vec<StatusEntry>,
    /// Commits ahead/behind the upstream tracking branch, if known.
    pub ahead: usize,
    pub behind: usize,
}

/// Build auth callbacks for an HTTPS remote using a GitHub PAT.
/// For GitHub, basic auth with the token as the username works for both
/// classic and fine-grained tokens.
fn auth_callbacks(token: Option<String>) -> RemoteCallbacks<'static> {
    let mut cb = RemoteCallbacks::new();
    cb.credentials(move |_url, username_from_url, _allowed| match &token {
        Some(tok) => Cred::userpass_plaintext(tok, ""),
        None => Cred::username(username_from_url.unwrap_or("git")),
    });
    cb
}

pub fn open(path: &Path) -> AppResult<Repository> {
    Ok(Repository::open(path)?)
}

/// Open an existing repo at `path`, or initialize a new one (with the scaffold
/// directories) if none exists yet.
pub fn open_or_init(path: &Path) -> AppResult<Repository> {
    if path.join(".git").exists() {
        return open(path);
    }
    std::fs::create_dir_all(path)?;
    let repo = Repository::init(path)?;
    // Seed the standard layout so the tree is browsable immediately.
    std::fs::create_dir_all(path.join("cves"))?;
    std::fs::create_dir_all(path.join("guides"))?;
    let readme = path.join("README.md");
    if !readme.exists() {
        std::fs::write(
            &readme,
            "# ExpRepo data\n\nCVE writeups and exploits managed by ExpRepo.\n",
        )?;
    }
    Ok(repo)
}

pub fn clone(url: &str, path: &Path, token: Option<String>) -> AppResult<Repository> {
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(auth_callbacks(token));
    let mut builder = RepoBuilder::new();
    builder.fetch_options(fo);
    Ok(builder.clone(url, path)?)
}

pub fn get_remote_url(repo: &Repository) -> Option<String> {
    repo.find_remote(REMOTE)
        .ok()
        .and_then(|r| r.url().ok().map(|s| s.to_string()))
}

pub fn set_remote_url(repo: &Repository, url: &str) -> AppResult<()> {
    if repo.find_remote(REMOTE).is_ok() {
        repo.remote_set_url(REMOTE, url)?;
    } else {
        repo.remote(REMOTE, url)?;
    }
    Ok(())
}

fn current_branch(repo: &Repository) -> Option<String> {
    let head = repo.head().ok()?;
    head.shorthand().ok().map(|s| s.to_string())
}

pub fn status(repo: &Repository) -> AppResult<RepoStatus> {
    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(true);
    let statuses = repo.statuses(Some(&mut opts))?;

    let mut entries = Vec::new();
    for s in statuses.iter() {
        let st = s.status();
        let path = s.path().unwrap_or("").to_string();
        let (state, staged) = if st.is_conflicted() {
            ("conflicted".to_string(), false)
        } else if st.intersects(git2::Status::INDEX_NEW) {
            ("new".to_string(), true)
        } else if st.intersects(git2::Status::INDEX_MODIFIED) {
            ("modified".to_string(), true)
        } else if st.intersects(git2::Status::INDEX_DELETED) {
            ("deleted".to_string(), true)
        } else if st.intersects(git2::Status::INDEX_RENAMED) {
            ("renamed".to_string(), true)
        } else if st.intersects(git2::Status::WT_NEW) {
            ("new".to_string(), false)
        } else if st.intersects(git2::Status::WT_MODIFIED) {
            ("modified".to_string(), false)
        } else if st.intersects(git2::Status::WT_DELETED) {
            ("deleted".to_string(), false)
        } else if st.intersects(git2::Status::WT_RENAMED) {
            ("renamed".to_string(), false)
        } else {
            ("typechange".to_string(), false)
        };
        entries.push(StatusEntry { path, state, staged });
    }

    let (ahead, behind) = ahead_behind(repo).unwrap_or((0, 0));

    Ok(RepoStatus {
        branch: current_branch(repo),
        remote_url: get_remote_url(repo),
        entries,
        ahead,
        behind,
    })
}

fn ahead_behind(repo: &Repository) -> AppResult<(usize, usize)> {
    let head = repo.head()?;
    let local_oid = head.target().ok_or_else(|| AppError::msg("no HEAD target"))?;
    let branch = head.shorthand()?;
    let upstream_ref = format!("refs/remotes/{REMOTE}/{branch}");
    let upstream = match repo.refname_to_id(&upstream_ref) {
        Ok(oid) => oid,
        Err(_) => return Ok((0, 0)),
    };
    Ok(repo.graph_ahead_behind(local_oid, upstream)?)
}

/// Stage everything and create a commit. Returns the new commit's short id.
pub fn commit_all(repo: &Repository, message: &str, name: &str, email: &str) -> AppResult<String> {
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;
    let sig = Signature::now(name, email)?;

    let parent_commit = match repo.head() {
        Ok(head) => head.target().and_then(|oid| repo.find_commit(oid).ok()),
        Err(_) => None,
    };
    let parents: Vec<&git2::Commit> = parent_commit.iter().collect();

    let oid = repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?;
    Ok(oid.to_string()[..7.min(oid.to_string().len())].to_string())
}

pub fn push(repo: &Repository, branch: &str, token: Option<String>) -> AppResult<()> {
    let mut remote = repo
        .find_remote(REMOTE)
        .map_err(|_| AppError::msg("no 'origin' remote configured"))?;
    let mut po = PushOptions::new();
    po.remote_callbacks(auth_callbacks(token));
    let refspec = format!("refs/heads/{branch}:refs/heads/{branch}");
    remote.push(&[refspec.as_str()], Some(&mut po))?;
    Ok(())
}

/// Fetch from origin and fast-forward the current branch. If the branches have
/// diverged, returns an error asking the user to resolve manually (v1 has no
/// merge UI).
pub fn pull(repo: &Repository, branch: &str, token: Option<String>) -> AppResult<()> {
    let mut remote = repo
        .find_remote(REMOTE)
        .map_err(|_| AppError::msg("no 'origin' remote configured"))?;
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(auth_callbacks(token));
    remote.fetch(&[branch], Some(&mut fo), None)?;

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
    let (analysis, _) = repo.merge_analysis(&[&fetch_commit])?;

    if analysis.is_up_to_date() {
        return Ok(());
    }
    if analysis.is_fast_forward() {
        let refname = format!("refs/heads/{branch}");
        let mut reference = repo.find_reference(&refname)?;
        reference.set_target(fetch_commit.id(), "fast-forward")?;
        repo.set_head(&refname)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        return Ok(());
    }
    Err(AppError::msg(
        "local and remote have diverged; resolve manually with an external git client",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("exprepo-repo-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn init_commit_yields_clean_status() {
        let dir = tmp();
        let repo = open_or_init(&dir).unwrap();
        std::fs::write(dir.join("a.txt"), "hello").unwrap();
        let short = commit_all(&repo, "init", "Tester", "t@example.com").unwrap();
        assert!(!short.is_empty());

        let st = status(&repo).unwrap();
        assert!(st.entries.is_empty(), "expected clean tree, got {:?}", st.entries);
        assert!(st.branch.is_some());

        // A new untracked file should show up as a single change.
        std::fs::write(dir.join("b.txt"), "x").unwrap();
        let st2 = status(&repo).unwrap();
        assert_eq!(st2.entries.len(), 1);
        assert_eq!(st2.entries[0].path, "b.txt");
        assert_eq!(st2.entries[0].state, "new");

        std::fs::remove_dir_all(&dir).ok();
    }
}
