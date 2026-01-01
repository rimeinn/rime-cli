use std::path::PathBuf;
use structopt::StructOpt;

mod download;
mod install;
mod package;
mod recipe;
mod rime_levers;

use download::{DownloadParams, download_recipe_package};
use install::install_recipe;
use recipe::RecipeInfo;
use rime_levers::{
    add_to_schema_list, build_binaries, setup_engine_traits, select_schema, apply_patch
};

#[derive(Debug, StructOpt)]
#[structopt(about = "Rime 配方管理器")]
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
        #[structopt(flatten)]
        download_params: DownloadParams,
    },
    /// 安裝配方
    Install {
        /// 要安裝的配方
        recipes: Vec<String>,
        #[structopt(flatten)]
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

    let args = SubCommands::from_args();
    log::debug!("參數: {:?}", args);

    match args {
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
            recipes, download_params
        } => {
            let recipes = recipes
                .iter()
                .map(|rx| RecipeInfo::from(rx.as_str()))
                .collect::<Vec<_>>();
            download_recipe_package(&recipes, download_params)?;
        }
        SubCommands::Install {
            recipes, download_params
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

    Ok(())
}
