use anyhow::Result;
use cat_file::cmd_cat_file;
use clap::Parser;
use git_object::GitObjectKind;
use hash_object::cmd_hash_object;
use init::cmd_init;
use log::cmd_log;
use std::path::PathBuf;

mod cat_file;
mod git_config;
mod git_object;
mod git_repository;
mod hash_object;
mod init;
mod log;

#[derive(Debug, clap::Parser)]
enum CLI {
    Add {
        path: PathBuf,
    },
    CatFile {
        kind: GitObjectKind,
        object: String,
    },
    CheckIgnore,
    Checkout,
    Commit,
    HashObject {
        // -w オプションとして使えるようにする
        #[arg(short)]
        write: bool,
        #[arg(short)]
        kind: GitObjectKind,
        path: PathBuf,
    },
    Init {
        path: PathBuf,
    },
    Log {
        object: String,
    },
    LsFiles,
    LsTree,
    RevParse,
    Rm,
    ShowRef,
    Status,
    Tag,
}

fn main() -> Result<()> {
    // cargo run -- --hoge fuga
    // -- はcargo runの引数とclapの引数を分けるために必要
    match CLI::try_parse()? {
        CLI::Add { .. } => todo!(),
        CLI::CatFile { kind, object } => cmd_cat_file(kind, object)?,
        CLI::CheckIgnore => todo!(),
        CLI::Checkout => todo!(),
        CLI::Commit => todo!(),
        CLI::HashObject { write, kind, path } => cmd_hash_object(write, kind, path)?,
        CLI::Init { path } => cmd_init(path)?,
        CLI::Log { object } => cmd_log(object)?,
        CLI::LsFiles => todo!(),
        CLI::LsTree => todo!(),
        CLI::RevParse => todo!(),
        CLI::Rm => todo!(),
        CLI::ShowRef => todo!(),
        CLI::Status => todo!(),
        CLI::Tag => todo!(),
    }

    // indexmap が入れた順番に取り出せる
    // `shift_insert` で指定した位置に挿入できる
    // let mut map = IndexMap::<&str, &str>::new();
    // map.insert("a", "1");

    // chrono 日付
    // let now = chrono::Local::now();
    Ok(())
}
// e26f7edc2238102ee185f7c852a9c5356b938b75
