use crate::{
    git_object::{object_read, GitObject, GitObjectKind},
    git_repository::repo_find,
};
use anyhow::Result;
use std::path::PathBuf;

pub fn cmd_ls_tree(tree: String, recursive: bool) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let gitdir = repo_find(&current_dir)?.gitdir;
    ls_tree(&gitdir, tree, recursive, &PathBuf::from(""))?;

    Ok(())
}

fn ls_tree(gitdir: &PathBuf, r#ref: String, recursive: bool, prefix: &PathBuf) -> Result<()> {
    let sha = r#ref.clone();
    let obj = object_read(gitdir, &sha)?;
    let GitObject::Tree(objects) = obj else {
        return Err(anyhow::anyhow!("Expected tree, got {:?}", obj));
    };
    for o in objects {
        let kind = o.file_type.kind();
        if !recursive || kind != GitObjectKind::Tree {
            println!(
                "{}{:0>4} {} {}\t{}",
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
