use anyhow::Result;
use cat_file::cmd_cat_file;
use checkout::cmd_checkout;
use clap::Parser;
use git_object::GitObjectKind;
use hash_object::cmd_hash_object;
use init::cmd_init;
use log::cmd_log;
use ls_tree::cmd_ls_tree;
use show_ref::cmd_show_ref;
use std::{env, path::PathBuf};
use tag::{cmd_ls_tag, cmd_tag};

mod cat_file;
mod checkout;
mod git_config;
mod git_object;
mod git_repository;
mod hash_object;
mod init;
mod log;
mod ls_tree;
mod show_ref;
mod tag;

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
    Checkout {
        commit: String,
        path: PathBuf,
    },
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
    LsTree {
        tree: String,
        #[arg(short)]
        recursive: bool,
    },
    RevParse,
    Rm,
    ShowRef,
    Status,
    LsTag,
    Tag {
        name: String,
        // -a or --annotate
        #[arg(short)]
        annotate: bool,
        object: String,
    },
}

fn parse() -> Result<CLI> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() == 2 && args[1] == "tag".to_string() {
        return Ok(CLI::LsTag);
    }
    CLI::try_parse_from(args).map_err(|e| e.into())
}

fn main() -> Result<()> {
    // cargo run -- --hoge fuga
    // -- はcargo runの引数とclapの引数を分けるために必要
    match parse()? {
        CLI::Add { .. } => todo!(),
        CLI::CatFile { kind, object } => cmd_cat_file(kind, object)?,
        CLI::CheckIgnore => todo!(),
        CLI::Checkout { commit, path } => cmd_checkout(commit, path)?,
        CLI::Commit => todo!(),
        CLI::HashObject { write, kind, path } => cmd_hash_object(write, kind, path)?,
        CLI::Init { path } => cmd_init(path)?,
        CLI::Log { object } => cmd_log(object)?,
        CLI::LsFiles => todo!(),
        CLI::LsTree { tree, recursive } => cmd_ls_tree(tree, recursive)?,
        CLI::RevParse => todo!(),
        CLI::Rm => todo!(),
        CLI::ShowRef => cmd_show_ref()?,
        CLI::Status => todo!(),
        CLI::LsTag => cmd_ls_tag()?,
        CLI::Tag {
            name,
            annotate,
            object,
        } => cmd_tag(name, annotate, object)?,
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
