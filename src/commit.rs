use crate::git_config::GitConfigUser;
use anyhow::{Ok, Result};
use std::path::{Path, PathBuf};

pub(crate) fn cmd_commit(message: Option<String>) -> Result<()> {
    let config_path = PathBuf::from(std::env::home_dir().unwrap()).join(".gitconfig");
    let conf = GitConfigUser::read(&config_path)?;
    println!("{:?}", conf.name_email());
    Ok(())
}
