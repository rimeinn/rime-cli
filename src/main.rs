use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod download;
mod install;
mod package;
mod recipe;
mod rime_levers;

use download::{download_recipe_package, DownloadParams};
use install::install_recipe;
use recipe::RecipeInfo;
use rime_levers::{
    add_to_schema_list, apply_patch, build_binaries, select_schema, setup_engine_traits,
};

#[derive(Debug, Parser)]
#[command(about, author, version, arg_required_else_help(true))]
struct Program {
    #[command(subcommand)]
    subcommands: Option<SubCommands>,
}

#[derive(Debug, Subcommand)]
enum SubCommands {
    /// 加入輸入方案列表
    Add {
        /// 要向列表中追加的輸入方案
        schemata: Vec<String>,
    },
    /// 構建輸入法固件
    Build,
    /// 部署輸入法固件到目標位置
    Deploy,
    /// 下載配方包
    Download {
        /// 要下載的配方包
        recipes: Vec<String>,
        #[command(flatten)]
        download_params: DownloadParams,
    },
    /// 安裝配方
    Install {
        /// 要安裝的配方
        recipes: Vec<String>,
        #[command(flatten)]
        download_params: DownloadParams,
    },
    /// 新建配方
    New {
        /// 配方名字
        _name: Option<String>,
    },
    /// 配置補丁
    Patch {
        /// 目標配置
        config: String,
        /// 紐
        key: String,
        /// 值
        value: String,
    },
    /// 選擇輸入方案
    Select {
        /// 選中的輸入方案
        schema: String,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Program::parse();

    if let Some(s) = args.subcommands {
        log::debug!("參數: {:?}", s);
        match s {
            SubCommands::Add { schemata } => {
                let current_path = PathBuf::from(".");
                setup_engine_traits(&current_path)?;
                add_to_schema_list(&schemata)?;
            }
            SubCommands::Build => {
                let current_path = PathBuf::from(".");
                setup_engine_traits(&current_path)?;
                build_binaries()?;
            }
            SubCommands::Download {
                recipes,
                download_params,
            } => {
                let recipes = recipes
                    .iter()
                    .map(|rx| RecipeInfo::from(rx.as_str()))
                    .collect::<Vec<_>>();
                download_recipe_package(&recipes, download_params)?;
            }
            SubCommands::Install {
                recipes,
                download_params,
            } => {
                let recipes = recipes
                    .iter()
                    .map(|rx| RecipeInfo::from(rx.as_str()))
                    .collect::<Vec<_>>();
                download_recipe_package(&recipes, download_params)?;
                for recipe in &recipes {
                    install_recipe(recipe)?;
                }
            }
            SubCommands::Patch { config, key, value } => {
                let current_path = PathBuf::from(".");
                setup_engine_traits(&current_path)?;
                apply_patch(&config, &key, &value)?;
            }
            SubCommands::Select { schema } => {
                select_schema(&schema)?;
            }
            _ => todo!("還沒做呢"),
        }
    }

    Ok(())
}
