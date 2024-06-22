use crate::{
    git_object::{GitObject, GitObjectKind},
    git_repository::repo_find,
};
use anyhow::Result;
use std::{fs::File, io::Read, path::PathBuf};

pub fn cmd_hash_object(write: bool, kind: GitObjectKind, path: PathBuf) -> Result<()> {
    let mut f = File::open(&path)?;
    let mut content = Vec::new();
    let size = f.read_to_end(&mut content)?;
    let gitdir = repo_find(&path)?.gitdir;

    let obj = GitObject {
        kind,
        size,
        content,
    };

    if write {
        obj.write(&gitdir)?;
    }

    println!("{}", obj.hash()?);
    Ok(())
}
