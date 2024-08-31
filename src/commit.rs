use crate::git_index::GitIndexEntry;
use crate::git_object::{FileType, GitObject, TreeObject};
use crate::git_repository::repo_find;
use crate::ls_files::index_read;
use crate::resolve::object_find;
use crate::status::branch_get_active;
use crate::{git_config::GitConfigUser, git_index::GitIndex};
use anyhow::{Ok, Result};
use chrono::Local;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::vec;

pub(crate) fn cmd_commit(message: Option<String>, index_path: Option<PathBuf>) -> Result<()> {
    let config_path = PathBuf::from(std::env::home_dir().unwrap()).join(".gitconfig");
    let conf = GitConfigUser::read(&config_path)?;

    let current_dir = std::env::current_dir()?;
    let gitdir = repo_find(&current_dir)?.gitdir;
    let index_path = index_path.unwrap_or_else(|| gitdir.join("index"));
    let index = index_read(&index_path)?;

    let tree = tree_from_index(&gitdir, index)?;
    let parent = object_find(&gitdir, "HEAD".to_string(), None)?;
    let Some(mut author) = conf.name_email() else {
        return Err(anyhow::anyhow!(""));
    };
    // <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>
    author = format!("{} {}", author, Local::now().format("%s %z"));
    let commit = GitObject::Commit {
        tree,
        parent: vec![parent],
        author: author.clone(),
        committer: author,
        message: message.unwrap_or_default(),
    };
    commit.write(&gitdir)?;

    match branch_get_active(&gitdir)? {
        Some(branch) => {
            let head_path = gitdir.join("refs").join("heads").join(branch);
            dbg!(&head_path);
            let mut head = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&head_path)?;
            writeln!(head, "{}", commit.hash()?)?;
        }
        None => {
            let head_path = gitdir.join("HEAD");
            let mut head = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&head_path)?;
            writeln!(head, "")?;
        }
    };
    Ok(())
}

#[derive(Debug)]
enum Value {
    File(GitIndexEntry),
    // dir, sha
    Dir(String, String),
}

impl Value {
    fn name(&self) -> String {
        match self {
            Value::File(e) => format!("File {}", e.name),
            Value::Dir(d, _) => format!("Dir {}", d),
        }
    }
}

fn debug_tree_map(tree_map: &HashMap<PathBuf, Vec<Value>>) {
    for (k, e) in tree_map {
        println!("{}", k.display());
        for v in e {
            println!("- {}", v.name());
        }
    }
}

fn tree_from_index(gitdir: &PathBuf, index: GitIndex) -> Result<String> {
    fn parent(p: PathBuf) -> PathBuf {
        if let Some(p) = p.parent() {
            p.to_path_buf()
        } else {
            PathBuf::new()
        }
    }

    let mut tree_map: HashMap<PathBuf, Vec<Value>> = HashMap::new();
    tree_map.insert(PathBuf::from(""), vec![]);
    for entry in index.entries {
        let original_path = PathBuf::from(&entry.name);
        let mut path = parent(original_path.clone()).clone();
        while path != PathBuf::new() {
            tree_map.entry(path.clone()).or_insert(vec![]);
            path = parent(path.clone());
        }
        // dbg!(&original_path);
        tree_map
            .entry(parent(original_path))
            .and_modify(|e| e.push(Value::File(entry)));
    }

    println!("1");
    debug_tree_map(&tree_map);

    // always encounter a given path before its parent
    let mut sorted_path = tree_map.keys().cloned().collect::<Vec<_>>();
    sorted_path.sort_by_key(|p| Reverse(p.to_string_lossy().len()));
    for path in sorted_path {
        let mut tree_vec = vec![];
        for val in tree_map.get(&path).unwrap() {
            let tree_object = match val {
                Value::File(entry) => TreeObject {
                    file_type: FileType::from_u16(entry.mode_type)?,
                    permission: (entry.mode_perms as u32).to_be_bytes().try_into().unwrap(),
                    path: entry.name.to_string().into(),
                    sha: entry.sha.to_string(),
                },
                Value::Dir(base, sha) => TreeObject {
                    file_type: FileType::Tree,
                    permission: [0; 4],
                    path: base.into(),
                    sha: sha.clone(),
                },
            };
            tree_vec.push(tree_object);
        }

        let tree = GitObject::Tree(tree_vec);
        tree.write(&gitdir)?;
        let sha = tree.hash()?;
        let parent = parent(path.clone());
        let base = path.strip_prefix(&parent).unwrap();
        assert_eq!(parent.join(base), path, "ng");
        tree_map
            .get_mut(&parent)
            .unwrap()
            .push(Value::Dir(base.to_string_lossy().to_string(), sha.clone()));
    }

    println!("2");
    debug_tree_map(&tree_map);
    // 文字列長降順でソートされているので .last() に root ("") が来る
    let Some(Value::Dir(_, root_sha)) = &tree_map.get(&PathBuf::from("")).unwrap().last() else {
        return Err(anyhow::anyhow!("unexpected Value::File"));
    };

    Ok(root_sha.to_string())
}
