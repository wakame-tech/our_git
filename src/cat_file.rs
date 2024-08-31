use crate::{
    git_object::{object_read, serialize_object, GitObjectKind},
    git_repository::repo_find,
    resolve::object_find,
};
use anyhow::Result;

pub fn cmd_cat_file(_kind: GitObjectKind, object_str: String) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let repo = repo_find(&current_dir)?.gitdir;
    let sha = object_find(&repo, object_str.clone(), None)?;
    let object = object_read(&repo, &sha)?;
    println!("{:?}", object);
    let bin = serialize_object(&object);
    let object_str = String::from_utf8_lossy(&bin);
    println!("{}", object_str);
    Ok(())
}
