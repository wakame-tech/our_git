use crate::{
    git_object::{object_read, GitObject},
    git_repository::repo_find,
};
use anyhow::Result;
use std::{collections::HashSet, path::PathBuf};

pub fn cmd_log(object_str: String) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let gitdir = repo_find(&current_dir)?.gitdir;
    println!("digraph wyaglog {{");
    println!("\tnode [shape=rect];");
    // TODO: HEADに対応
    let sha = object_read(&gitdir, &object_str)?.hash()?;
    log_graphviz(&gitdir, sha, &mut HashSet::new())?;
    println!("}}");
    Ok(())
}

fn log_graphviz(gitdir: &PathBuf, sha: String, seen: &mut HashSet<String>) -> Result<()> {
    if seen.contains(&sha) {
        return Ok(());
    }
    seen.insert(sha.clone());
    let GitObject::Commit {
        parent, message, ..
    } = object_read(gitdir, &sha)?
    else {
        return Err(anyhow::anyhow!("Not a commit {}", sha));
    };
    let short_hash = &sha[..=7];
    let message = message.replace("\\", "\\\\");
    let message = message.replace("\"", "\\\"");
    let message = message.split("\n").collect::<Vec<_>>()[0];

    println!(
        "\t\"c_{}\" [label=\"{}\\n{}\", shape=rect];",
        sha, short_hash, message
    );
    for p in parent {
        println!("\tc_{} -> c_{};", sha, p);
        log_graphviz(gitdir, p, seen)?;
    }
    Ok(())
}
