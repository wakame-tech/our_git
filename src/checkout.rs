use anyhow::{Ok, Result};
use std::{fs, path::PathBuf};

use crate::{
    git_object::{object_read, GitObject, TreeOject},
    git_repository::repo_find,
};

pub fn cmd_checkout(commit: String, path: PathBuf) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let gitdir = repo_find(&current_dir)?.gitdir;
    let tree = match object_read(&gitdir, commit.as_str())? {
        GitObject::Commit { tree, .. } => object_read(&gitdir, tree.as_str())?,
        tree @ GitObject::Tree(_) => tree,
        _ => anyhow::bail!("commit or tree object expected"),
    };
    let GitObject::Tree(tree_vec) = tree else {
        anyhow::bail!("tree object expected");
    };

    if !path.exists() {
        anyhow::bail!("path does not exist");
    }
    if !path.is_dir() {
        anyhow::bail!("path is not a directory");
    }
    if path.read_dir()?.count() > 0 {
        anyhow::bail!("path is not empty");
    }

    fs::create_dir_all(&path)?;

    tree_checkout(&gitdir, &tree_vec, &path)?;

    Ok(())
}

fn tree_checkout(gitdir: &PathBuf, tree_vec: &[TreeOject], path: &PathBuf) -> Result<()> {
    for tree_obj in tree_vec {
        let obj = object_read(gitdir, &tree_obj.sha)?;
        let obj_path = path.join(&tree_obj.path);
        match obj {
            GitObject::Blob { content } => {
                fs::write(&obj_path, content)?;
            }
            GitObject::Tree(objects) => {
                fs::create_dir(&obj_path)?;
                tree_checkout(gitdir, &objects, &obj_path)?;
            }
            _ => anyhow::bail!("blob or tree object expected"),
        }
    }
    Ok(())
}
