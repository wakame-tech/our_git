use crate::git_repository::repo_find;
use anyhow::Result;
use std::{
    collections::BTreeMap,
    fs::{read_dir, File},
    io::Read,
    path::PathBuf,
};

pub fn cmd_show_ref() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let gitdir = repo_find(&current_dir)?.gitdir;
    let refs_dir = gitdir.join("refs");
    let refs = ref_list(&gitdir, &refs_dir)?;
    show_ref(refs, true)
}

fn ref_resolve(gitdir: &PathBuf, ref_path: &PathBuf) -> Result<String> {
    let mut f = File::open(ref_path)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    let buf = buf.trim().to_string();
    if buf.starts_with("ref: ") {
        let ref_path = buf.trim_start_matches("ref: ");
        ref_resolve(gitdir, &gitdir.join(ref_path))
    } else {
        Ok(buf)
    }
}

pub fn ref_list(gitdir: &PathBuf, current: &PathBuf) -> Result<BTreeMap<PathBuf, String>> {
    let mut refs = BTreeMap::new();
    for entry in read_dir(current)? {
        let path = entry?.path();
        if path.is_dir() {
            refs.extend(ref_list(gitdir, &path)?);
        } else {
            let relative_path = path.strip_prefix(gitdir)?.to_path_buf();
            refs.insert(relative_path, ref_resolve(gitdir, &path)?);
        }
    }
    Ok(refs)
}

pub fn show_ref(refs: BTreeMap<PathBuf, String>, with_hash: bool) -> Result<()> {
    for (k, v) in refs {
        if with_hash {
            println!("{} -> {}", k.display(), v)
        } else {
            println!("{}", k.display())
        }
    }
    Ok(())
}
