use anyhow::Result;
use clap::Parser;
use init::cmd_init;
use std::path::PathBuf;

mod git_config;
mod git_object;
mod git_repository;
mod init;

#[derive(Debug, clap::Parser)]
enum CLI {
    Add { path: PathBuf },
    CatFile,
    CheckIgnore,
    Checkout,
    Commit,
    HashObject,
    Init { path: PathBuf },
    Log,
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
        CLI::CatFile => todo!(),
        CLI::CheckIgnore => todo!(),
        CLI::Checkout => todo!(),
        CLI::Commit => todo!(),
        CLI::HashObject => todo!(),
        CLI::Init { path } => cmd_init(path)?,
        CLI::Log => todo!(),
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
