use std::io;

use anstyle::AnsiColor;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use dialoguer::{theme::ColorfulTheme, FuzzySelect};

use crate::{
    spdx::{display_license_ids, Licenses},
    template::{fill_license_forms, interact_write_template, write_template, Template},
    util::errors::{Error, LictoolResult},
};

#[derive(Parser, Debug)]
#[command(author, version,styles=get_styles())]
#[clap(arg_required_else_help = true)]
pub struct Cli {
    /// A field that holds the specific subcommand to be executed.
    #[clap(subcommand)]
    subcommand: CliCommand,
}

impl Cli {
    /// Asynchronously executes the command specified in the CLI
    /// structure.
    ///
    /// # Returns
    ///
    /// * `LictoolResult<()>` - The result of executing the command,
    ///   wrapped in the `LictoolResult` type.
    ///
    /// # Errors
    ///
    /// This function will return an error if the execution of the
    /// command fails.
    pub async fn exec_command(&self) -> LictoolResult<()> {
        match &self.subcommand {
            CliCommand::Completions {
                shell,
            } => {
                generate(
                    shell.to_owned(),
                    &mut Cli::command(),
                    env!("CARGO_BIN_NAME"),
                    &mut io::stdout().lock(),
                );
                Ok(())
            }
            CliCommand::Init {
                path,
            } => {
                let licenses = Licenses::new().await?;
                let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select a license")
                    .items(&licenses.body)
                    .max_length(7)
                    .interact_opt()?;
                let license = &licenses.body[selection.unwrap_or(0)];
                let mut details = license.details().await?;
                let mut template = fill_license_forms(&mut details, &ColorfulTheme::default())?;
                Ok(interact_write_template(path, &mut template)?)
            }
            CliCommand::List {
                deprecated,
                supported,
                osi_approved,
                fsf_libre,
            } => {
                let licenses = Licenses::new().await?;
                let mut filtered =
                    licenses.filter_by(*deprecated, *supported, *osi_approved, *fsf_libre);
                display_license_ids(&mut filtered)
            }
            CliCommand::Add {
                license_id,
                owner,
                email,
                repo,
                year,
                path,
            } => {
                let licenses = Licenses::new().await?;
                if let Some(license) = licenses
                    .body
                    .iter()
                    .find(|lic| lic.to_string() == *license_id)
                {
                    let details = license.details().await?;
                    Ok(write_template(
                        path,
                        &mut Template {
                            license_text: details.license_text,
                            year: *year,
                            owner: owner.clone(),
                            repo: repo.clone(),
                            email: email.clone(),
                        },
                    )?)
                } else {
                    Err(Error::NotFound)?
                }
            }
            CliCommand::Info {
                license_id,
            } => {
                let licenses = Licenses::new().await?;
                if let Some(license) = licenses
                    .body
                    .iter()
                    .find(|lic| lic.to_string() == *license_id)
                {
                    let details = license.details().await?;
                    println!("{}", details);
                } else {
                    Err(Error::NotFound)?
                }
                Ok(())
            }
        }
    }
}

#[derive(Subcommand, Debug)]
/// Available commands
enum CliCommand {
    /// Initializes a license, prompting for details to fill
    /// placeholders
    Init {
        #[clap(short, long)]
        #[clap(default_value_t = String::from("LICENSE.md"))]
        path: String,
    },
    /// Add a license in the current directory without prompting for
    /// individual details
    Add {
        license_id: String,
        #[arg(short, long, alias = "author")]
        owner: Option<String>,
        #[arg(short, long)]
        email: Option<String>,
        #[arg(short, long)]
        repo: Option<String>,
        #[arg(short, long)]
        year: Option<i32>,
        #[clap(short, long)]
        #[clap(default_value_t = String::from("LICENSE.md"))]
        path: String,
    },
    /// Lists all available licenses
    List {
        /// Only deprecated
        #[arg(short, long)]
        deprecated: bool,
        #[arg(short, long)]
        /// Only supported
        supported: bool,
        #[arg(short, long)]
        /// Only OSI Approved
        osi_approved: bool,
        #[arg(short, long)]
        /// Only FSF Free/Libre
        fsf_libre: bool,
    },
    /// Get info about license
    Info { license_id: String },
    /// Generate completion scripts for your shell
    Completions {
        #[clap(value_enum)]
        shell: Shell,
    },
}

/// Retrieves the styles to be used in the command-line interface
/// (CLI) output.
///
/// # Returns
///
/// * `clap::builder::Styles` - The styles configured for usage and
///   header display in the CLI, including
fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(AnsiColor::Cyan))),
        )
        .header(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(AnsiColor::Magenta))),
        )
        .literal(anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(AnsiColor::Green))))
        .invalid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(AnsiColor::Red))),
        )
        .error(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(AnsiColor::Red))),
        )
        .valid(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(AnsiColor::Green))),
        )
        .placeholder(anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(AnsiColor::Yellow))))
}
