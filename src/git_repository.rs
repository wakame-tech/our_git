use crate::git_config::GitConfig;
use anyhow::Result;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub fn repo_path(repo: &GitRepository, paths: Vec<PathBuf>) -> PathBuf {
    paths
        .into_iter()
        .fold(repo.gitdir.clone(), |acc, p| acc.join(p))
}

pub fn repo_file(
    repo: &GitRepository,
    paths: Vec<PathBuf>,
    mkdir: Option<bool>,
) -> Result<Option<PathBuf>> {
    // Python: *path[:-1]
    // Rust: paths[..n].to_vec()
    if let Some(_) = repo_dir(repo, paths[..paths.len() - 1].to_vec(), mkdir)? {
        Ok(Some(repo_path(repo, paths)))
    } else {
        Ok(None)
    }
}

pub fn repo_dir(
    repo: &GitRepository,
    paths: Vec<PathBuf>,
    mkdir: Option<bool>,
) -> Result<Option<PathBuf>> {
    let mkdir = mkdir.unwrap_or(false);
    let path: PathBuf = repo_path(repo, paths);

    if path.exists() {
        if path.is_dir() {
            return Ok(Some(path));
        } else {
            return Err(anyhow::anyhow!("Not a directory {:?}", path));
        }
    }

    if mkdir {
        fs::create_dir_all(&path)?;
        Ok(Some(path))
    } else {
        Ok(None)
    }
}

pub fn repo_create(path: &PathBuf) -> Result<GitRepository> {
    let repo = GitRepository::new(path.clone(), Some(true))?;
    if repo.worktree.exists() {
        if !repo.worktree.is_dir() {
            return Err(anyhow::anyhow!("Not a directory {:?}", repo.worktree));
        }
        if repo.gitdir.exists() && repo.gitdir.read_dir()?.count() > 0 {
            return Err(anyhow::anyhow!(
                "Git dir does not exists of not empty {:?}",
                repo.gitdir
            ));
        }
    }
    fs::create_dir_all(&repo.worktree)?;

    repo_dir(&repo, vec![PathBuf::from("branches")], Some(true))?;
    repo_dir(&repo, vec![PathBuf::from("objects")], Some(true))?;
    repo_dir(
        &repo,
        vec![PathBuf::from("refs"), PathBuf::from("tags")],
        Some(true),
    )?;
    repo_dir(
        &repo,
        vec![PathBuf::from("refs"), PathBuf::from("heads")],
        Some(true),
    )?;

    // ["description"][:-1] は [] になって ./test/.git になるがそうなって欲しくない
    if let Some(path) = repo_file(&repo, vec![PathBuf::from("description")], Some(false))? {
        let mut f = OpenOptions::new().write(true).create(true).open(path)?;
        f.write(b"Unnamed repository; edit this file 'description' to name the repository.\n")?;
    }

    if let Some(path) = repo_file(&repo, vec![PathBuf::from("HEAD")], Some(false))? {
        let mut f = OpenOptions::new().write(true).create(true).open(path)?;
        f.write(b"ref: refs/heads/master\n")?;
    }

    if let Some(path) = repo_file(&repo, vec![PathBuf::from("config")], Some(false))? {
        let conf = GitConfig::default();
        conf.write(&path)?;
    }
    Ok(repo)
}

pub struct GitRepository {
    worktree: PathBuf,
    gitdir: PathBuf,
}

impl GitRepository {
    pub fn new(path: PathBuf, force: Option<bool>) -> Result<Self> {
        let force = force.unwrap_or(false);
        let gitdir = path.join(".git");

        if !(force || gitdir.is_dir()) {
            return Err(anyhow::anyhow!("not a git repository: {:?}", path));
        }

        let repo = Self {
            worktree: path,
            gitdir,
        };

        if let Some(path) = repo_file(&repo, vec![PathBuf::from("config")], Some(false))? {
            if !force {
                let conf = GitConfig::read(&path)?;
                if conf.repository_format_version != 0 {
                    return Err(anyhow::anyhow!(
                        "Unsupported repositoryformatversion {:?}",
                        conf.repository_format_version
                    ));
                }
            }
        }

        Ok(repo)
    }
}
