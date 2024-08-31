use std::{
    collections::HashSet,
    fs::{self, File},
    io::Read,
    os::macos::fs::MetadataExt,
    path::PathBuf,
};

use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::{
    git_index::{GitIndex, GitIndexEntry},
    git_object::GitObject,
    git_repository::repo_find,
    ls_files::index_read,
};

pub fn cmd_add(path_vec: &[PathBuf], index_path: Option<PathBuf>) -> Result<()> {
    let working_dir = std::env::current_dir()?;
    let gitdir = repo_find(&working_dir)?.gitdir;
    let index_path = index_path.unwrap_or_else(|| gitdir.join("index"));

    let mut index = if index_path.exists() {
        index_read(&index_path)?
    } else {
        GitIndex::new(2, vec![])
    };

    let path_set = path_vec
        .iter()
        .map(|p| {
            if p.is_absolute() {
                p.strip_prefix(&working_dir).unwrap()
            } else {
                p
            }
        })
        .filter(|p| p.exists())
        .collect::<HashSet<_>>();

    index
        .entries
        .retain(|e| !path_set.contains(PathBuf::from(e.name.to_string()).as_path()));

    for path in path_vec {
        let mut buf = vec![];
        // todo: directoryに対応
        let mut f = File::open(path)?;
        f.read_to_end(&mut buf)?;
        let blob = GitObject::Blob { content: buf };
        blob.write(&gitdir)?;
        let sha = blob.hash()?;
        let stat = fs::metadata(path)?;
        let ctime = DateTime::<Utc>::from(stat.created()?);
        let mtime = DateTime::<Utc>::from(stat.modified()?);
        let entry = GitIndexEntry {
            ctime,
            mtime,
            dev: stat.st_dev() as u32,
            ino: stat.st_ino() as u32,
            mode_type: (stat.st_mode() >> 12) as u16,
            mode_perms: (stat.st_mode() & 0o777) as u16,
            uid: stat.st_uid(),
            gid: stat.st_gid(),
            fsize: stat.st_size() as usize,
            sha,
            flag_assume_valid: stat.st_flags() >> 15 != 0,
            flag_stage: ((stat.st_flags() >> 12) & 0b0011) as u16,
            name: path.to_string_lossy().to_string(),
        };
        index.entries.push(entry);
    }
    index.entries.sort_by(|a, b| a.name.cmp(&b.name));
    let res = index.serialize()?;
    std::fs::write(&index_path, res)?;

    Ok(())
}
