use crate::{
    git_object::{object_read, GitObject, GitObjectKind},
    show_ref::ref_resolve,
};
use anyhow::Result;
use regex::Regex;
use std::{path::PathBuf, sync::LazyLock};

static PAT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9A-Fa-f]{4,40}$").unwrap());

fn object_resolve(gitdir: &PathBuf, object: String) -> Result<Vec<String>> {
    if &object == "HEAD" {
        return Ok(vec![ref_resolve(gitdir, &PathBuf::from("HEAD"))?]);
    }

    let mut candidates = vec![];
    if PAT.is_match(&object) {
        let prefix = &object[..2];
        let suffix = &object[2..];
        let dir = gitdir.join("objects").join(prefix);
        let results = dir
            .read_dir()?
            .into_iter()
            .filter_map(|e| {
                let Ok(e) = e else {
                    return None;
                };
                let path = e.path();
                if !path.is_file() {
                    return None;
                }
                if !path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with(suffix)
                {
                    return None;
                }
                let name = e.path().file_name().unwrap().to_str().unwrap().to_string();
                let sha = format!("{}{}", prefix, name);
                Some(sha)
            })
            .collect::<Vec<_>>();
        candidates.extend(results);
    }

    let tag_path = PathBuf::from("refs").join("tags").join(&object);
    if let Ok(sha) = ref_resolve(gitdir, &tag_path) {
        candidates.push(sha);
    };

    let branch_path = PathBuf::from("refs").join("heads").join(&object);
    if let Ok(sha) = ref_resolve(gitdir, &branch_path) {
        candidates.push(sha);
    };

    Ok(candidates)
}

pub(crate) fn object_find(
    gitdir: &PathBuf,
    object: String,
    format: Option<GitObjectKind>,
) -> Result<String> {
    let candidates = object_resolve(gitdir, object.clone())?;
    let sha = match candidates.len() {
        0 => anyhow::bail!("object not found: {}", object),
        1 => candidates[0].clone(),
        _ => anyhow::bail!("ambiguous object: {}", object),
    };
    let obj = object_read(gitdir, &sha)?;
    if Some(GitObjectKind::Tree) == format {
        if let GitObject::Commit { tree, .. } = obj {
            return Ok(tree);
        } else if let GitObject::Tree(_) = obj {
            return Ok(sha);
        };
        return Err(anyhow::anyhow!("commit or tree object expected"));
    };

    if let GitObject::Tag { object, .. } = obj {
        object_find(gitdir, object, format)
    } else {
        Ok(sha)
    }
}
