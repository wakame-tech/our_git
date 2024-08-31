use crate::{git_index::GitIndex, git_repository::repo_find};
use anyhow::Result;
use std::{fs::File, io::Read, path::PathBuf};

pub(crate) fn index_read(index_path: &PathBuf) -> Result<GitIndex> {
    let mut index_file = File::open(index_path)?;
    let mut content = vec![];
    index_file.read_to_end(&mut content)?;
    GitIndex::parse(content)
}

pub(crate) fn cmd_ls_files(verbose: bool, index_path: Option<PathBuf>) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let gitdir = repo_find(&current_dir)?.gitdir;
    let index_path = index_path.unwrap_or_else(|| gitdir.join("index"));
    let index = if !index_path.exists() {
        GitIndex::new(2, vec![])
    } else {
        index_read(&index_path)?
    };
    for entry in &index.entries {
        if verbose {
            println!("{}", entry.name);
        } else {
            println!("{:?}", entry);
        }
    }
    Ok(())
}
