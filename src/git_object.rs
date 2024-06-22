use anyhow::Result;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
};

#[derive(Debug, Clone)]
pub enum GitObjectKind {
    Blob,
    Commit,
    Tag,
    Tree,
}

impl FromStr for GitObjectKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<GitObjectKind, Self::Err> {
        GitObjectKind::from_str(s).ok_or(anyhow::anyhow!("Invalid header"))
    }
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
    pub kind: GitObjectKind,
    pub size: usize,
    pub content: Vec<u8>,
}

impl GitObject {
    pub fn hash(&self) -> Result<String> {
        let data = serialize_object(self);
        let result = [
            self.kind.as_str().to_string().into_bytes(),
            vec![b' '],
            format!("{}", self.size).into_bytes(),
            vec![0],
            data,
        ]
        .concat();
        let sha = Sha1::digest(&result);
        let sha = hex::encode(sha);
        Ok(sha)
    }

    pub fn write(&self, gitdir: &PathBuf) -> Result<()> {
        let data = serialize_object(self);
        let result = [
            self.kind.as_str().to_string().into_bytes(),
            vec![b' '],
            format!("{}", self.size).into_bytes(),
            vec![0],
            data,
        ]
        .concat();
        let sha = self.hash()?;
        let path = gitdir.join("objects").join(&sha[..2]).join(&sha[2..]);
        fs::create_dir_all(path.parent().unwrap())?;
        if !path.exists() {
            let f = File::create(&path)?;
            let mut zipped = ZlibEncoder::new(f, Compression::default());
            zipped.write_all(&result)?;
        }
        Ok(())
    }
}

pub fn serialize_object(obj: &GitObject) -> Vec<u8> {
    match obj.kind {
        GitObjectKind::Blob => obj.content.clone(),
        _ => todo!(),
    }
}

pub fn deserialize_object(data: &[u8]) -> GitObject {
    todo!()
}

pub fn object_read(gitdir: &PathBuf, sha: &str) -> Result<GitObject> {
    // https://docs.rs/flate2/latest/flate2/read/struct.ZlibDecoder.html
    let path = gitdir.join("objects").join(&sha[..2]).join(&sha[2..]);
    anyhow::ensure!(path.is_file(), "{} is not a file", path.display());

    dbg!(&path);
    let f = File::open(path)?;
    let mut bin = Vec::new();
    ZlibDecoder::new(f).read_to_end(&mut bin)?;

    let space_at = bin.iter().position(|&b| b == b' ').unwrap();
    let header = String::from_utf8(bin[..space_at].to_vec())?;
    let kind = GitObjectKind::from_str(&header).ok_or(anyhow::anyhow!("Invalid header"))?;
    let null_at = bin.iter().position(|&b| b == 0).unwrap();
    let size: usize = String::from_utf8(bin[space_at + 1..null_at].to_vec())?.parse()?;
    let content = bin[null_at + 1..].to_vec();
    anyhow::ensure!(size == content.len(), "Size mismatch");
    Ok(GitObject {
        kind,
        size,
        content,
    })
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
