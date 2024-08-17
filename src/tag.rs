use crate::{
    git_object::{GitObject, GitObjectKind},
    git_repository::repo_find,
    resolve::object_find,
    show_ref::{ref_list, show_ref},
};
use anyhow::{Ok, Result};
use std::fs;

pub fn cmd_ls_tag() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let gitdir = repo_find(&current_dir)?.gitdir;
    let tags_dir = gitdir.join("refs").join("tags");
    let refs = ref_list(&gitdir, &tags_dir)?;
    show_ref(refs, true)
}

pub fn cmd_tag(name: String, annotate: bool, object: String) -> Result<()> {
    let gitdir = repo_find(&std::env::current_dir()?)?.gitdir;
    anyhow::ensure!(
        object_find(&gitdir, object.clone(), None).is_ok(),
        "Failed to resolve '{}' as a valid ref",
        object
    );
    let tagger = "tagger <tagger1919@example.com>".to_string();
    let message = "tag message".to_string();
    if annotate {
        create_tag_object(name, object, tagger, message)
    } else {
        create_lightweight_tag(name, object)
    }
}

fn create_tag_object(name: String, object: String, tagger: String, message: String) -> Result<()> {
    let sha = object;
    let tag = GitObject::Tag {
        object: sha,
        kind: GitObjectKind::Commit,
        tag: name.clone(),
        tagger,
        message,
    };
    let gitdir = repo_find(&std::env::current_dir()?)?.gitdir;
    tag.write(&gitdir)?;

    let tag_sha = tag.hash()?;
    let tags_dir = gitdir.join("refs").join("tags");
    fs::create_dir_all(&tags_dir)?;
    fs::write(&tags_dir.join(&name), tag_sha.clone() + "\n")?;

    create_ref(name, tag_sha)
}

fn create_lightweight_tag(ref_name: String, object: String) -> Result<()> {
    create_ref(ref_name, object)
}

fn create_ref(ref_name: String, object: String) -> Result<()> {
    let gitdir = repo_find(&std::env::current_dir()?)?.gitdir;
    let tags_dir = gitdir.join("refs").join("tags");
    let ref_path = tags_dir.join(ref_name);
    fs::write(&ref_path, object + "\n")?;

    Ok(())
}
