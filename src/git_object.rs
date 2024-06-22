use anyhow::Result;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

#[derive(Debug)]
pub enum GitObjectKind {
    Blob,
    Commit,
    Tag,
    Tree,
}

impl GitObjectKind {
    pub fn as_str(&self) -> &str {
        match self {
            GitObjectKind::Blob => "blob",
            GitObjectKind::Commit => "commit",
            GitObjectKind::Tag => "tag",
            GitObjectKind::Tree => "tree",
        }
    }

    pub fn from_str(s: &str) -> Option<GitObjectKind> {
        match s {
            "blob" => Some(GitObjectKind::Blob),
            "commit" => Some(GitObjectKind::Commit),
            "tag" => Some(GitObjectKind::Tag),
            "tree" => Some(GitObjectKind::Tree),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct GitObject {
    kind: GitObjectKind,
    size: usize,
    content: Vec<u8>,
}

pub fn serialize_object(obj: &GitObject) -> Vec<u8> {
    Vec::new()
}

pub fn deserialize_object(data: &[u8]) -> GitObject {
    todo!()
}

pub fn object_read(gitdir: &PathBuf, sha: &str) -> Result<GitObject> {
    // https://docs.rs/flate2/latest/flate2/read/struct.ZlibDecoder.html
    let path = gitdir.join("objects").join(&sha[..2]).join(&sha[2..]);
    anyhow::ensure!(path.is_file(), "{} is not a file", path.display());

    let f = File::open(path)?;
    let mut bin = Vec::new();
    ZlibDecoder::new(f).read(&mut bin)?;

    let space_at = bin.iter().position(|&b| b == b' ').unwrap();
    let header = String::from_utf8(bin[..space_at].to_vec())?;
    let kind = GitObjectKind::from_str(&header).ok_or(anyhow::anyhow!("Invalid header"))?;
    let null_at = bin.iter().skip(space_at).position(|&b| b == 0).unwrap();
    let size: usize = String::from_utf8(bin[space_at..null_at].to_vec())?.parse()?;
    let content = bin[null_at..].to_vec();
    anyhow::ensure!(size == content.len(), "Size mismatch");
    Ok(GitObject {
        kind,
        size,
        content,
    })
}

pub fn object_write(gitdir: &PathBuf, obj: &GitObject) -> Result<String> {
    let data = serialize_object(obj);
    let result = [
        obj.kind.as_str().to_string().into_bytes(),
        vec![b' '],
        format!("{}", obj.size).into_bytes(),
        vec![0],
        data,
    ]
    .concat();
    let sha = sha1::Sha1::digest(&result);
    let sha = hex::encode(sha);
    let path = gitdir.join("objects").join(&sha[..2]).join(&sha[2..]);
    fs::create_dir_all(path.parent().unwrap())?;
    if !path.exists() {
        let f = File::create(&path)?;
        let mut zipped = ZlibEncoder::new(f, Compression::default());
        zipped.write_all(&result)?;
    }
    Ok(sha)
}

#[cfg(test)]
mod tests {
    use sha1::digest::Digest;

    #[test]
    fn a() {
        let result = "hello world".to_string().into_bytes();
        let hash = sha1::Sha1::digest(&result);
        let hash = hex::encode(hash);
        assert_eq!(hash, "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed")
    }
}
