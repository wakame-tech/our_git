use crate::{
    git_object::{object_read, GitObject, GitObjectKind},
    git_repository::repo_find,
    resolve::object_find,
};
use anyhow::Result;
use std::path::PathBuf;

pub fn cmd_ls_tree(object: String, recursive: bool) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let gitdir = repo_find(&current_dir)?.gitdir;
    let sha = object_find(&gitdir, object, Some(GitObjectKind::Tree))?;
    ls_tree(&gitdir, sha, recursive, &PathBuf::from(""))?;

    Ok(())
}

fn ls_tree(gitdir: &PathBuf, sha: String, recursive: bool, prefix: &PathBuf) -> Result<()> {
    let sha = sha.clone();
    let obj = object_read(gitdir, &sha)?;
    let GitObject::Tree(objects) = obj else {
        return Err(anyhow::anyhow!("Expected tree, got {:?}", obj));
    };
    for o in objects {
        let kind = o.file_type.kind();
        if !recursive || kind != GitObjectKind::Tree {
            println!(
                "{}{:?} {} {}\t{}",
                o.file_type.as_str(),
                o.permission,
                kind.as_str(),
                o.sha,
                prefix.join(o.path).display(),
            );
        } else {
            ls_tree(gitdir, o.sha, recursive, &prefix.join(o.path))?;
        }
    }
    Ok(())
}
