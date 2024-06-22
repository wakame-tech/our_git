use crate::{
    git_object::{object_read, serialize_object, GitObjectKind},
    git_repository::repo_find,
};
use anyhow::Result;

pub fn cmd_cat_file(_kind: GitObjectKind, object_str: String) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let repo = repo_find(&current_dir)?.gitdir;
    let object = object_read(&repo, object_str.as_str())?;
    let object_str = String::from_utf8(serialize_object(&object))?;
    println!("{}", object_str);
    Ok(())
}
