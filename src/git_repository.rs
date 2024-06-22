use crate::git_config::GitConfig;
use anyhow::Result;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

fn touch_file(file_path: &PathBuf, content: &[u8]) -> Result<()> {
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .open(file_path)?;
    f.write(content)?;
    Ok(())
}

pub fn repo_create(path: &PathBuf) -> Result<GitRepository> {
    let repo = GitRepository::new(path.clone(), Some(true))?;
    if repo.worktree.exists() {
        anyhow::ensure!(
            repo.worktree.is_dir(),
            "Not a directory {:?}",
            repo.worktree
        );
        anyhow::ensure!(
            !(repo.gitdir.exists() && repo.gitdir.read_dir()?.count() > 0),
            "Git dir does not exists or not empty {:?}",
            repo.gitdir
        );
    }
    fs::create_dir_all(&repo.worktree)?;

    fs::create_dir(&repo.gitdir)?;
    for dir in [
        repo.gitdir.join("branches"),
        repo.gitdir.join("objects"),
        repo.gitdir.join("refs").join("tags"),
        repo.gitdir.join("refs").join("heads"),
    ] {
        if dir.exists() {
            anyhow::ensure!(dir.is_dir(), "Not a directory {:?}", path);
        }
        fs::create_dir_all(&dir)?;
    }

    // ["description"][:-1] は [] になって ./test/.git になるがそうなって欲しくない
    touch_file(
        &repo.gitdir.join("description"),
        b"Unnamed repository; edit this file 'description' to name the repository.\n",
    )?;
    touch_file(&repo.gitdir.join("HEAD"), b"ref: refs/heads/master\n")?;
    let conf = GitConfig::default();
    conf.write(&repo.gitdir.join("config"))?;
    Ok(repo)
}

pub fn repo_find(path: &Path) -> Result<GitRepository> {
    let mut current = Some(path);
    while let Some(path) = current {
        if path.join(".git").is_dir() {
            return GitRepository::new(path.to_path_buf(), None);
        }
        current = path.parent();
    }
    anyhow::bail!("git directory not found");
}

pub struct GitRepository {
    worktree: PathBuf,
    gitdir: PathBuf,
}

impl GitRepository {
    pub fn new(path: PathBuf, force: Option<bool>) -> Result<Self> {
        let force = force.unwrap_or(false);
        let gitdir = path.join(".git");

        anyhow::ensure!(force || gitdir.is_dir(), "not a git repository: {:?}", path);

        let repo = Self {
            worktree: path,
            gitdir,
        };

        let path = &repo.gitdir.join("config");
        if !force {
            let conf = GitConfig::read(&path)?;
            anyhow::ensure!(
                conf.repository_format_version == 0,
                "Unsupported repositoryformatversion {:?}",
                conf.repository_format_version
            )
        }

        Ok(repo)
    }
}
