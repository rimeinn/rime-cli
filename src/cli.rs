use std::sync::LazyLock;

use clap::{ArgAction, Args, Parser, Subcommand, builder::Styles};

use crate::{download::DownloadParams, fl};

static HELP_HEADING: LazyLock<String> = LazyLock::new(|| fl!("clap-command"));
static ARG_HELP_HEADING: LazyLock<String> = LazyLock::new(|| fl!("clap-options"));
static ARG_HELP_HEADING_MUST: LazyLock<String> = LazyLock::new(|| fl!("clap-argument"));
static HELP_TEMPLATE: LazyLock<String> = LazyLock::new(|| {
    format!(
        "\
{{before-help}}{{about-with-newline}}
{}{}:{} {{usage}}

{{all-args}}{{after-help}}\
    ",
        Styles::default().get_usage().render(),
        fl!("clap-usage"),
        Styles::default().get_usage().render_reset()
    )
});

#[derive(Debug, Parser)]
#[command(
    version,
    about = fl!("clap-about"),
    max_term_width = 80,
    subcommand_help_heading = &**HELP_HEADING,
    subcommand_value_name = &**HELP_HEADING,
    next_help_heading = &**ARG_HELP_HEADING,
    disable_version_flag = true,
    disable_help_flag = true,
    disable_help_subcommand = true,
    override_usage = format!(
        "{}rime{} [{}] [{}]",
        Styles::default().get_literal().render(),
        Styles::default().get_literal().render_reset(),
        fl!("clap-command"),
        fl!("clap-argument")
    ),
    help_template = &*HELP_TEMPLATE,
)]

pub struct Program {
    #[command(flatten)]
    pub options: Options,
    #[command(subcommand)]
    pub subcommands: Option<SubCommands>,
}

#[derive(Debug, Args)]
pub struct Options {
    /// Print version
    #[arg(short, long, action = ArgAction::Version, help = fl!("clap-version-help"))]
    version: Option<bool>,
    /// Print help
    #[arg(long, short, global = true, action = ArgAction::Help, help = fl!("clap-help"))]
    help: Option<bool>,
}

#[derive(Debug, Subcommand)]
pub enum SubCommands {
    /// Add to schema list
    #[command(about = fl!("clap-add-help"), help_template = &*HELP_TEMPLATE)]
    #[command(next_help_heading = &**ARG_HELP_HEADING)]
    Add {
        /// Schema(ta) to append to schema list
        #[arg(help = fl!("clap-add-schemata-help"))]
        schemata: Vec<String>,
    },
    /// Remove from schema list
    #[command(
        visible_alias = "del",
        visible_alias = "rm",
        about = fl!("clap-remove-help"),
        help_template = &*HELP_TEMPLATE
    )]
    #[command(next_help_heading = &**ARG_HELP_HEADING)]
    Remove {
        /// Schema(ta) to remove from schema list
        #[arg(help = fl!("clap-remove-schemata-help"))]
        schemata: Vec<String>,
    },
    /// Build binaries
    #[command(about = fl!("clap-build-help"), help_template = &*HELP_TEMPLATE)]
    #[command(next_help_heading = &**ARG_HELP_HEADING)]
    Build,
    /// Deploy binaries to target place
    #[command(about = fl!("clap-deploy-help"), help_template = &*HELP_TEMPLATE)]
    #[command(next_help_heading = &**ARG_HELP_HEADING)]
    Deploy,
    /// Download recipe package(s)
    #[command(about = fl!("clap-download-help"), help_template = &*HELP_TEMPLATE)]
    #[command(next_help_heading = &**ARG_HELP_HEADING)]
    Download {
        /// Recipe(s) to donwload
        #[arg(
            help = fl!("clap-download-recipes-help"),
            help_heading = &**ARG_HELP_HEADING_MUST
        )]
        recipes: Vec<String>,
        #[command(flatten)]
        download_params: DownloadParams,
    },
    /// Install recipe(s)
    #[command(about = fl!("clap-install-help"), help_template = &*HELP_TEMPLATE)]
    #[command(next_help_heading = &**ARG_HELP_HEADING)]
    Install {
        /// Recipe(s) to install
        #[arg(
            help = fl!("clap-install-recipes-help"),
            help_heading = &**ARG_HELP_HEADING_MUST
        )]
        recipes: Vec<String>,
        #[command(flatten)]
        download_params: DownloadParams,
    },
    /// Create new recipe
    #[command(about = fl!("clap-new-help"), help_template = &*HELP_TEMPLATE)]
    #[command(next_help_heading = &**ARG_HELP_HEADING)]
    New {
        /// Recipe name
        #[arg(help = fl!("clap-new-recipe-help"))]
        _name: Option<String>,
    },
    /// Apply patch for config
    #[command(about = fl!("clap-patch-help"), help_template = &*HELP_TEMPLATE)]
    #[command(next_help_heading = &**ARG_HELP_HEADING)]
    Patch {
        /// Target config
        #[arg(help = fl!("clap-patch-config-help"))]
        config: String,
        /// Key
        #[arg(help = fl!("clap-patch-key-help"))]
        key: String,
        /// Value
        #[arg(help = fl!("clap-patch-value-help"))]
        value: String,
    },
    /// Select schema
    #[command(about = fl!("clap-select-help"), help_template = &*HELP_TEMPLATE)]
    #[command(next_help_heading = &**ARG_HELP_HEADING)]
    Select {
        /// Selected schema
        #[arg(help = fl!("clap-select-schema-help"))]
        schema: String,
    },
}