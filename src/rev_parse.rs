use crate::{git_object::GitObjectKind, git_repository::repo_find, resolve::object_find};
use anyhow::Result;

pub(crate) fn cmd_rev_parse(format: Option<GitObjectKind>, object: String) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let gitdir = repo_find(&current_dir)?.gitdir;
    let sha = object_find(&gitdir, object, format)?;
    println!("{}", sha);
    Ok(())
}
