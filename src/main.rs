use clap::{CommandFactory, FromArgMatches};
use clap_i18n_richformatter::CommandI18nExt;
use std::path::PathBuf;

mod cli;
mod download;
mod install;
mod lang;
mod package;
mod recipe;
mod rime_levers;

use download::download_recipe_package;
use i18n_embed::{DesktopLanguageRequester, Localizer};
use install::install_recipe;
use lang::LANGUAGE_LOADER;
use recipe::RecipeInfo;
use rime_levers::{
    add_to_schema_list, apply_patch, build_binaries, select_schema, setup_engine_traits,
};

use crate::{cli::{Program, SubCommands}, rime_levers::{available_schemata, remove_from_schema_list, selected_schemata}};

fn init_localizer() {
    let localizer = crate::lang::localizer();
    let requested_languages = DesktopLanguageRequester::requested_languages();

    if let Err(error) = localizer.select(&requested_languages) {
        eprintln!("Error while loading languages for library_fluent {error}");
    }

    // Windows Terminal doesn't support bidirectional (BiDi) text, and renders the isolate characters incorrectly.
    // This is a temporary workaround for https://github.com/microsoft/terminal/issues/16574
    // TODO: this might break BiDi text, though we don't support any writing system depends on that.
    LANGUAGE_LOADER.set_use_isolating(false);
}

fn main() -> anyhow::Result<()> {
    init_localizer();
    env_logger::init();

    let args = parse_args();

    let subcmd = args.subcommands;
    if subcmd.is_none() {
        Program::command().print_help()?;
        return Ok(());
    }

    let subcmd = subcmd.unwrap();
    log::debug!("參數: {:?}", subcmd);

    match subcmd {
        SubCommands::Add { schemata } => {
            let current_path = PathBuf::from(".");
            setup_engine_traits(&current_path)?;
            add_to_schema_list(&schemata)?;
        }
        SubCommands::Remove { schemata } => {
            let current_path = PathBuf::from(".");
            setup_engine_traits(&current_path)?;
            remove_from_schema_list(&schemata)?;
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
        SubCommands::List { selected, available} => {
            let current_path = PathBuf::from(".");
            setup_engine_traits(&current_path)?;
            if selected {
                let selected_schemata = selected_schemata()?;
                println!("{:?}", selected_schemata);
            } else if available {
                let available_schemata = available_schemata()?;
                println!("{:?}", available_schemata);
            }
        }
        SubCommands::Select { schema } => {
            select_schema(&schema)?;
        }
        _ => todo!("還沒做呢"),
    }

    Ok(())
}

fn parse_args() -> Program {
    let matches = Program::command().get_matches_i18n();

    let program = match Program::from_arg_matches(&matches).map_err(|e| {
        let mut cmd = Program::command();
        e.format(&mut cmd)
    }) {
        Ok(program) => program,
        Err(e) => e.exit(),
    };

    program
}
