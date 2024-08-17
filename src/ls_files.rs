use crate::{git_index::GitIndex, git_repository::repo_find};
use anyhow::Result;
use std::{fs::File, io::Read, path::PathBuf};

fn index_read(gitdir: &PathBuf) -> Result<GitIndex> {
    let index_path = gitdir.join("index");
    if !index_path.exists() {
        return Ok(GitIndex::new(2, vec![]));
    }
    let mut index_file = File::open(index_path)?;
    let mut content = vec![];
    index_file.read_to_end(&mut content)?;
    GitIndex::parse(content)
}

pub(crate) fn cmd_ls_files(verbose: bool) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let gitdir = repo_find(&current_dir)?.gitdir;
    let index = index_read(&gitdir)?;
    for entry in &index.entries {
        if verbose {
            println!("{}", entry.name);
        } else {
            println!("{:?}", entry);
        }
    }
    Ok(())
}
