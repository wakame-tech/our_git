use anyhow::Result;
use std::{collections::HashSet, path::PathBuf};

use crate::{git_index::GitIndex, git_repository::repo_find, ls_files::index_read};

pub(crate) fn cmd_rm(path_vec: Vec<PathBuf>, index_path: Option<PathBuf>) -> Result<()> {
    let working_dir = std::env::current_dir()?;
    let gitdir = repo_find(&working_dir)?.gitdir;
    let index_path = index_path.unwrap_or_else(|| gitdir.join("index"));
    let index = if index_path.exists() {
        index_read(&index_path)?
    } else {
        GitIndex::new(2, vec![])
    };

    let path_set = path_vec
        .iter()
        .map(|p| {
            let relative_path = if p.is_absolute() {
                p.strip_prefix(&working_dir).unwrap()
            } else {
                p
            };
            relative_path.to_string_lossy().to_string()
        })
        .collect::<HashSet<_>>();

    let new_entries = index
        .entries
        .into_iter()
        .filter(|entry| !path_set.contains(&entry.name))
        .collect::<Vec<_>>();

    let new_index = GitIndex {
        entries: new_entries,
        ..index
    };
    let res = new_index.serialize()?;
    std::fs::write(&index_path, res)?;
    Ok(())
}
