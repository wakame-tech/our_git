use add::cmd_add;
use anyhow::Result;
use cat_file::cmd_cat_file;
use checkout::cmd_checkout;
use clap::Parser;
use git_object::GitObjectKind;
use hash_object::cmd_hash_object;
use init::cmd_init;
use log::cmd_log;
use ls_files::cmd_ls_files;
use ls_tree::cmd_ls_tree;
use rev_parse::cmd_rev_parse;
use rm::cmd_rm;
use show_ref::cmd_show_ref;
use status::cmd_status;
use std::{env, path::PathBuf};
use tag::{cmd_ls_tag, cmd_tag};

mod add;
mod cat_file;
mod checkout;
mod git_config;
mod git_index;
mod git_object;
mod git_repository;
mod hash_object;
mod init;
mod log;
mod ls_files;
mod ls_tree;
mod resolve;
mod rev_parse;
mod rm;
mod show_ref;
mod status;
mod tag;

#[derive(Debug, clap::Parser)]
enum CLI {
    Add {
        path_vec: Vec<PathBuf>,
        #[arg(short)]
        index_path: Option<PathBuf>,
    },
    CatFile {
        kind: GitObjectKind,
        object: String,
    },
    CheckIgnore,
    Checkout {
        object: String,
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
    LsFiles {
        #[arg(short)]
        verbose: bool,
        #[arg(short)]
        index_path: Option<PathBuf>,
    },
    LsTree {
        tree: String,
        #[arg(short)]
        recursive: bool,
    },
    RevParse {
        #[arg(short)]
        kind: Option<GitObjectKind>,
        object: String,
    },
    Rm {
        path_vec: Vec<PathBuf>,
        #[arg(short)]
        index_path: Option<PathBuf>,
    },
    ShowRef,
    Status {
        #[arg(short)]
        index_path: Option<PathBuf>,
    },
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
        CLI::Add {
            path_vec,
            index_path,
        } => cmd_add(&path_vec, index_path)?,
        CLI::CatFile { kind, object } => cmd_cat_file(kind, object)?,
        CLI::CheckIgnore => todo!(),
        CLI::Checkout {
            object: commit,
            path,
        } => cmd_checkout(commit, path)?,
        CLI::Commit => todo!(),
        CLI::HashObject { write, kind, path } => cmd_hash_object(write, kind, path)?,
        CLI::Init { path } => cmd_init(path)?,
        CLI::Log { object } => cmd_log(object)?,
        CLI::LsFiles {
            verbose,
            index_path,
        } => cmd_ls_files(verbose, index_path)?,
        CLI::LsTree { tree, recursive } => cmd_ls_tree(tree, recursive)?,
        CLI::RevParse { kind, object } => cmd_rev_parse(kind, object)?,
        CLI::Rm {
            path_vec,
            index_path,
        } => {
            anyhow::ensure!(!path_vec.is_empty(), "path_vec must not be empty");
            cmd_rm(&path_vec, index_path)?
        }
        CLI::ShowRef => cmd_show_ref()?,
        CLI::Status { index_path } => cmd_status(index_path)?,
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
