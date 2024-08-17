use anyhow::Result;
use bytes::Buf;
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub(crate) struct GitIndexEntry {
    pub(crate) ctime: DateTime<Utc>,
    pub(crate) mtime: DateTime<Utc>,
    pub(crate) dev: u32,
    pub(crate) ino: u32,
    pub(crate) mode_type: u32,
    pub(crate) mode_perms: u16,
    pub(crate) uid: u32,
    pub(crate) gid: u32,
    pub(crate) fsize: usize,
    pub(crate) sha: String,
    pub(crate) flag_assume_valid: bool,
    pub(crate) flag_stage: u16,
    pub(crate) name: String,
}

#[derive(Debug)]
pub(crate) struct GitIndex {
    pub(crate) version: u32,
    pub(crate) entries: Vec<GitIndexEntry>,
}

impl GitIndex {
    pub(crate) fn new(version: u32, entries: Vec<GitIndexEntry>) -> Self {
        Self { version, entries }
    }

    pub(crate) fn parse(raw: Vec<u8>) -> Result<Self> {
        let mut header = raw.as_slice();
        let signature = header.get(0..4).unwrap();
        header.advance(4);
        anyhow::ensure!(signature.chunk() == b"DIRC", "invalid signature");
        let version = header.get_u32();
        anyhow::ensure!(version == 2, "unsupported version: {}", version);
        let count = header.get_u32();

        let mut entries = vec![];
        for _ in 0..count {
            let ctime_s = header.get_u32();
            let ctime_ns = header.get_u32();
            let ctime = DateTime::from_timestamp(ctime_s as i64, ctime_ns).unwrap();
            let mtime_s = header.get_u32();
            let mtime_ns = header.get_u32();
            let mtime = DateTime::from_timestamp(mtime_s as i64, mtime_ns).unwrap();
            let dev = header.get_u32();
            let ino = header.get_u32();
            let unused = header.get_u16();
            anyhow::ensure!(unused == 0, "unused field is not zero");
            let mode = header.get_u16();
            let mode_type = (mode >> 12) as u32;
            anyhow::ensure!(
                vec![0b1000, 0b1010, 0b1110].contains(&mode_type),
                "invalid mode"
            );
            let mode_perms = mode & 0b0000_0001_1111_1111;
            let uid = header.get_u32();
            let gid = header.get_u32();
            let fsize = header.get_u32() as usize;
            let sha = header
                .take(20)
                .chunk()
                .iter()
                .map(|&c| format!("{:02x}", c))
                .collect::<String>();
            header.advance(20);
            let flags = header.get_u16();
            let flag_assume_valid = flags >> 15 != 0;
            let flag_extended = ((flags >> 14) & 0b01) != 0;
            anyhow::ensure!(!flag_extended, "flag_extended is not supported");
            let flag_stage = (flags >> 12) & 0b0011;
            let mut name_length = flags & 0x0fff;
            let raw_name = if name_length < 0xfff {
                anyhow::ensure!(
                    header.get(name_length as usize).unwrap() == &0u8,
                    "name is not null-terminated"
                );
                let raw_name = header.take(name_length as usize);
                raw_name.chunk().to_vec()
            } else {
                println!("Notice: Name is {} bytes long", name_length);
                let null_idx = header
                    .chunk()
                    .iter()
                    .position(|&c| c == 0u8)
                    .expect("null not found");
                name_length = null_idx as u16;
                let raw_name = header.take(null_idx);
                raw_name.chunk().to_vec()
            };
            let name = String::from_utf8(raw_name)?;
            // 1 byte for null-terminated
            header.advance(name_length as usize + 1);
            // align to 8 bytes
            let len = 62 + name_length as usize + 1;
            let pad = if len % 8 == 0 { 0 } else { 8 - len % 8 };
            header.advance(pad);

            let entry = GitIndexEntry {
                ctime,
                mtime,
                dev,
                ino,
                mode_type,
                mode_perms,
                uid,
                gid,
                fsize,
                sha,
                flag_assume_valid,
                flag_stage,
                name,
            };
            entries.push(entry);
        }
        Ok(Self::new(version, entries))
    }
}
