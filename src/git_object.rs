use anyhow::Result;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use indexmap::IndexMap;
use sha1::{Digest, Sha1};
use std::{
    fs::{self, File},
    io::{Cursor, Read, Seek, SeekFrom, Write},
    path::PathBuf,
    str::FromStr,
};

#[derive(Debug, Clone, PartialEq, Eq)]
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
        match dbg!(s) {
            "blob" => Some(GitObjectKind::Blob),
            "commit" => Some(GitObjectKind::Commit),
            "tag" => Some(GitObjectKind::Tag),
            "tree" => Some(GitObjectKind::Tree),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum GitObject {
    Blob {
        content: Vec<u8>,
    },
    Commit {
        // SHA-1 hash
        tree: String,
        // 2つより多くなるのかわからん
        parent: Vec<String>,
        author: String,
        committer: String,
        // gpgsig: String,
        message: String,
    },
    Tag {
        object: String,
        kind: GitObjectKind,
        tag: String,
        tagger: String,
        message: String,
    },
    Tree(Vec<TreeOject>),
}

#[derive(Debug, Clone)]
pub struct TreeOject {
    pub file_type: FileType,
    pub permission: String,
    pub path: PathBuf,
    pub sha: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileType {
    Tree,
    RegularFile,
    SymbolicLink,
    Submodule,
}

impl FileType {
    pub fn as_str(&self) -> &str {
        match self {
            FileType::Tree => "04",
            FileType::RegularFile => "10",
            FileType::SymbolicLink => "12",
            FileType::Submodule => "16",
        }
    }

    pub fn kind(&self) -> GitObjectKind {
        match self {
            FileType::Tree => GitObjectKind::Tree,
            FileType::RegularFile => GitObjectKind::Blob,
            FileType::SymbolicLink => GitObjectKind::Blob,
            FileType::Submodule => GitObjectKind::Commit,
        }
    }
}

impl TryFrom<&[u8]> for FileType {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<FileType> {
        match value {
            b"04" => Ok(FileType::Tree),
            b"10" => Ok(FileType::RegularFile),
            b"12" => Ok(FileType::SymbolicLink),
            b"16" => Ok(FileType::Submodule),
            _ => anyhow::bail!("Invalid file type {:#0x?}", value),
        }
    }
}

impl GitObject {
    fn kind(&self) -> GitObjectKind {
        match self {
            GitObject::Blob { .. } => GitObjectKind::Blob,
            GitObject::Commit { .. } => GitObjectKind::Commit,
            GitObject::Tag { .. } => GitObjectKind::Tag,
            _ => todo!(),
        }
    }

    pub fn hash(&self) -> Result<String> {
        let data = serialize_object(self);
        let result = [
            self.kind().as_str().to_string().into_bytes(),
            vec![b' '],
            format!("{}", data.len()).into_bytes(),
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
            self.kind().as_str().to_string().into_bytes(),
            vec![b' '],
            format!("{}", data.len()).into_bytes(),
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
    match obj {
        GitObject::Blob { content, .. } => content.clone(),
        GitObject::Commit {
            tree,
            parent,
            author,
            committer: comitter,
            message,
            ..
        } => {
            let dct = IndexMap::from_iter(vec![
                ("tree".to_string(), vec![tree.clone()]),
                ("parent".to_string(), parent.clone()),
                ("author".to_string(), vec![author.clone()]),
                ("committer".to_string(), vec![comitter.clone()]),
                // (Some("gpgsig".to_string()), vec![gpgsig.clone()]),
                ("message".to_string(), vec![message.clone()]),
            ]);
            serialize_indexmap(dct).unwrap()
        }
        GitObject::Tree(_) => tree_serialize(obj).unwrap(),
        GitObject::Tag {
            object,
            kind,
            tag,
            tagger,
            message,
            ..
        } => {
            let dct = IndexMap::from_iter(vec![
                ("object".to_string(), vec![object.clone()]),
                ("type".to_string(), vec![kind.as_str().to_string()]),
                ("tag".to_string(), vec![tag.clone()]),
                ("tagger".to_string(), vec![tagger.clone()]),
                ("message".to_string(), vec![message.clone()]),
            ]);
            serialize_indexmap(dct).unwrap()
        }
    }
}

pub fn deserialize_object(data: &[u8]) -> GitObject {
    todo!()
}

// commitをparseする
pub fn parse_commit<'a>(
    data: &[u8],
    start: usize,
    dct: &mut IndexMap<String, Vec<String>>,
) -> Result<()> {
    let spc = data[start..]
        .iter()
        .position(|&data| data == b' ')
        .map(|i| (start + i) as i32)
        .unwrap_or(-1);
    let nl = data[start..]
        .iter()
        .position(|&data| data == b'\n')
        .map(|i| (start + i) as i32)
        .unwrap_or(-1);

    if (spc < 0) || nl < spc {
        assert!(nl == start as i32);
        dct.insert(
            "message".to_string(),
            vec![String::from_utf8(data[start + 1..].to_vec())?],
        );
        return Ok(());
    }
    let spc = spc as usize;

    let key = String::from_utf8(data[start..spc].to_vec())?;
    let mut end = start;
    loop {
        end = data[end + 1..]
            .iter()
            .position(|&data| data == b'\n')
            .map(|i| end + 1 + i)
            .unwrap();
        // 32: ord(' '), 半角スペースのASCIIコード
        if data[end + 1] != ' ' as u8 {
            break;
        }
    }
    let value = String::from_utf8(data[spc + 1..end].to_vec())?.replace("\n ", "\n");

    if let Some(e) = dct.get_mut(&key) {
        // 要素があったらpush
        e.push(value);
    } else {
        // なかったらinsert
        dct.insert(key, vec![value]);
    }

    parse_commit(data, end + 1, dct)
}

pub fn serialize_indexmap(kvml: IndexMap<String, Vec<String>>) -> Result<Vec<u8>> {
    let mut ret = String::new();
    for (k, val) in kvml.iter() {
        if k == "message" {
            continue;
        }
        for v in val.iter() {
            ret += format!("{} {}\n", k, v.replace("\n", "\n ")).as_str();
        }
    }
    let message = kvml["message"][0].to_string();
    ret += format!("\n{}\n", message).as_str();
    Ok(ret.as_bytes().to_vec())
}

pub fn object_read(gitdir: &PathBuf, sha: &str) -> Result<GitObject> {
    // https://docs.rs/flate2/latest/flate2/read/struct.ZlibDecoder.html
    let path = gitdir.join("objects").join(&sha[..2]).join(&sha[2..]);
    anyhow::ensure!(path.is_file(), "{} is not a file", path.display());

    dbg!(&path);
    let f = File::open(path)?;
    let mut bin = Vec::new();
    ZlibDecoder::new(f).read_to_end(&mut bin)?;

    // git fetch --refetch --no-auto-gc
    let space_at = bin.iter().position(|&b| b == b' ').unwrap();
    let header = String::from_utf8(bin[..space_at].to_vec())?;
    let null_at = bin.iter().position(|&b| b == 0).unwrap();
    let size: usize = String::from_utf8(bin[space_at + 1..null_at].to_vec())?.parse()?;
    let content = bin[null_at + 1..].to_vec();

    match header.as_str() {
        "blob" => {
            anyhow::ensure!(size == content.len(), "Size mismatch");
            Ok(GitObject::Blob { content })
        }
        "commit" => {
            let mut dct = IndexMap::new();
            parse_commit(&content, 0, &mut dct)?;
            Ok(GitObject::Commit {
                tree: dct["tree"][0].clone(),
                parent: dct.get("parent").unwrap_or(&vec![]).clone(),
                author: dct["author"][0].clone(),
                committer: dct["committer"][0].clone(),
                // gpgsig: dct[&Some("gpgsig".to_string())][0].clone(),
                message: dct["message"][0].clone(),
            })
        }
        "tree" => tree_parse(&content),
        _ => anyhow::bail!("object-read() not support {}", header),
    }
}

pub fn tree_parse(data: &[u8]) -> Result<GitObject> {
    let mut objects = Vec::new();
    let mut cursor = Cursor::new(data);
    while cursor.position() < data.len() as u64 {
        let size = cursor
            .get_ref()
            .iter()
            .skip(cursor.position() as usize)
            .take_while(|&&b| b != 0x20)
            .count();
        let mut buf: Vec<u8> = vec![0; size];
        cursor.read_exact(&mut buf)?;
        let (file_type, permission) = if size == 5 {
            let file_type = FileType::try_from([b'0', buf[0]].as_slice())?;
            let permission = String::from_utf8(buf[2..5].to_vec())?;
            (file_type, permission)
        } else {
            let file_type = FileType::try_from(&buf[0..2])?;
            let permission = String::from_utf8(buf[2..6].to_vec())?;
            (file_type, permission)
        };

        cursor.seek(SeekFrom::Current(1))?;
        let size = cursor
            .get_ref()
            .iter()
            .skip(cursor.position() as usize)
            .take_while(|&&b| b != 0x00)
            .count();
        let mut buf = vec![0; size];
        cursor.read_exact(&mut buf)?;
        let path = PathBuf::from(String::from_utf8(buf)?);

        cursor.seek(SeekFrom::Current(1))?;
        let mut buf = vec![0; 20];
        cursor.read_exact(&mut buf)?;
        let sha = buf
            .iter()
            .map(|&c| format!("{:02x}", c))
            .collect::<String>();
        let tree = TreeOject {
            file_type,
            permission,
            path,
            sha,
        };
        objects.push(tree);
    }
    Ok(GitObject::Tree(objects))
}

fn tree_serialize(obj: &GitObject) -> Result<Vec<u8>> {
    let GitObject::Tree(mut objects) = obj.clone() else {
        anyhow::bail!("Invalid object");
    };
    objects.sort_by_key(|o| {
        if o.file_type == FileType::RegularFile {
            o.path.display().to_string()
        } else {
            o.path.display().to_string() + "/"
        }
    });
    let mut ret = vec![];
    for o in objects {
        ret.extend(o.file_type.as_str().as_bytes());
        ret.extend(o.permission.as_bytes());
        ret.push(b' ');
        ret.extend(o.path.display().to_string().as_bytes());
        ret.push(0x00);
        ret.extend(o.sha.as_bytes());
    }
    Ok(ret)
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
