use crate::{
    git_index::GitIndex,
    git_object::{object_read, FileType, GitObject, GitObjectKind},
    git_repository::repo_find,
    ls_files::index_read,
    resolve::object_find,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use ignore::Walk;
use std::{
    collections::{BTreeSet, HashMap},
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

fn branch_get_active(gitdir: &PathBuf) -> Result<Option<String>> {
    let head = gitdir.join("HEAD");
    let mut content = String::new();
    File::open(head)?.read_to_string(&mut content)?;
    if content.starts_with("ref: refs/heads/") {
        Ok(Some(
            content.trim_start_matches("ref: refs/heads/").to_string(),
        ))
    } else {
        Ok(None)
    }
}

fn cmd_status_branch(gitdir: &PathBuf) -> Result<()> {
    if let Some(branch) = branch_get_active(gitdir)? {
        println!("On branch {}", branch);
    } else {
        println!("HEAD detached at {}", "TODO");
    }
    Ok(())
}

fn tree_to_dict(
    gitdir: &PathBuf,
    r#ref: String,
    prefix: PathBuf,
    tree: &mut HashMap<PathBuf, String>,
) -> Result<()> {
    let tree_sha = object_find(gitdir, r#ref, Some(GitObjectKind::Tree))?;
    let GitObject::Tree(items) = object_read(gitdir, &tree_sha)? else {
        return Err(anyhow::anyhow!("not a tree object"));
    };
    for item in items {
        let path = prefix.join(&item.path);
        if item.file_type == FileType::Tree {
            tree_to_dict(gitdir, item.sha, path, tree)?;
        } else {
            tree.insert(path, item.sha);
        }
    }
    Ok(())
}

fn cmd_status_head_index(gitdir: &PathBuf, index: &GitIndex) -> Result<()> {
    println!("Changes to be committed:");
    let mut head = HashMap::new();
    tree_to_dict(gitdir, "HEAD".to_string(), PathBuf::new(), &mut head)?;
    for entry in &index.entries {
        let path = PathBuf::from(&entry.name);
        if let Some(sha) = head.get(&path) {
            if *sha != entry.sha {
                println!("\tmodified: {}", path.display());
            }
            head.remove(&path);
        } else {
            println!("\tadded: {}", path.display());
        }
    }
    for path in head.keys() {
        println!("\tdeleted: {}", path.display());
    }

    Ok(())
}

fn cmd_status_index_worktree(workdir: &PathBuf, index: &GitIndex) -> Result<()> {
    println!("Changes not staged for commit:");
    let mut rel_paths = BTreeSet::new();
    for e in Walk::new(workdir) {
        let e = e?;
        let rel_path = e.path().strip_prefix(workdir)?;
        if rel_path.is_file() {
            rel_paths.insert(rel_path.to_path_buf());
        }
    }
    for entry in &index.entries {
        let full_path = workdir.join(&entry.name);
        if let Ok(stat) = fs::metadata(&full_path) {
            if entry.ctime != DateTime::<Utc>::from(stat.created()?)
                || entry.mtime != DateTime::<Utc>::from(stat.modified()?)
            {
                let mut file = File::open(&full_path)?;
                let mut content = vec![];
                file.read_to_end(&mut content)?;
                let blob = GitObject::Blob { content };
                let new_sha = blob.hash()?;
                if new_sha != entry.sha {
                    println!("\tmodified: {}", entry.name);
                }
            }
        } else {
            println!("\tdeleted: {}", entry.name);
        }
        rel_paths.remove(&PathBuf::from(&entry.name));
    }

    println!("");
    println!("Untracked files:");
    for f in rel_paths {
        println!("\t{}", f.display());
    }
    Ok(())
}

pub(crate) fn cmd_status(index_path: Option<PathBuf>) -> Result<()> {
    let workdir = std::env::current_dir()?;
    let gitdir = repo_find(&workdir)?.gitdir;
    let index_path = index_path.unwrap_or_else(|| gitdir.join("index"));
    let index = if !index_path.exists() {
        GitIndex::new(2, vec![])
    } else {
        index_read(&index_path)?
    };
    cmd_status_branch(&gitdir)?;
    cmd_status_head_index(&gitdir, &index)?;
    println!("");
    cmd_status_index_worktree(&workdir, &index)?;
    Ok(())
}
