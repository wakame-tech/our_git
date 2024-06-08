use crate::git_repository::repo_create;
use anyhow::Result;
use std::path::PathBuf;

pub fn cmd_init(path: PathBuf) -> Result<()> {
    repo_create(&path)?;
    Ok(())
}
