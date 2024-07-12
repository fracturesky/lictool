use std::{fmt::Display, fs, mem::take, path::Path};

use anyhow::anyhow;
use chrono::{Datelike, Local};
use color_print::cprintln;
use dialoguer::{
    theme::{ColorfulTheme, Theme},
    Input,
};

use super::util::errors::Error;
use crate::{
    consts::{EMAIL, OWNER, REPO, YEAR},
    spdx::LicenseDetails,
    util::{errors::LictoolResult, git::GitConfig},
};

/// A struct representing a template for a license.
///
/// This struct holds fields for various components of a license
/// template, such as the license text, year, owner, repository, and
/// email.
#[derive(Debug, Default)]
pub struct Template {
    /// A string containing the text of the license.
    pub license_text: String,
    /// An optional integer representing the year of the license.
    pub year: Option<i32>,
    /// An optional string containing the owner's name.
    pub owner: Option<String>,
    /// An optional string containing the repository name.
    pub repo: Option<String>,
    /// An optional string containing the owner's email address.
    pub email: Option<String>,
}

impl Template {
    /// Renders the license template as a string.
    ///
    /// This function processes the fields of the `Template` struct,
    /// replacing placeholders with the actual values of `year`,
    /// `owner`, `repo`, and `email`, and returns the resulting
    /// string.
    ///
    /// # Returns
    ///
    /// A `String` containing the rendered license template.
    ///
    /// # Example
    ///
    /// ```
    /// let mut template = Template {
    ///     license_text: "This software is licensed under the terms of the LICENSE file.".to_string(),
    ///     year: Some(2024),
    ///     owner: Some("Alice".to_string()),
    ///     repo: Some("example_repo".to_string()),
    ///     email: Some("alice@example.com".to_string()),
    /// };
    /// let rendered = template.render();
    /// println!("{}", rendered);
    /// ```
    fn render(&mut self) -> String {
        let mut res = take(&mut self.license_text);
        if let Some(year) = self.year {
            YEAR.iter()
                .for_each(|&word| res = res.replace(word, &year.to_string()));
        }
        if let Some(owner) = &self.owner {
            OWNER
                .iter()
                .for_each(|&word| res = res.replace(word, owner));
        }
        if let Some(repo) = &self.repo {
            REPO.iter().for_each(|&word| res = res.replace(word, repo));
        }

        if let Some(email) = &self.email {
            EMAIL
                .iter()
                .for_each(|&word| res = res.replace(word, email));
        }
        res
    }
}

/// Fills a license template form with the provided license details
/// and theme.
///
/// This function takes mutable references to `LicenseDetails` and a
/// `Theme` object, fills the license template with the specified
/// details, and returns the resulting `Template`.
///
/// # Arguments
///
/// * `details` - A mutable reference to `LicenseDetails` containing
///   the license information.
/// * `theme` - A reference to a `Theme` trait object that customizes
///   the template appearance.
///
/// # Returns
///
/// A `LictoolResult` wrapping the `Template` struct containing
/// the filled template.
///
/// # Example
///
/// ```
/// let mut details = LicenseDetails { /* initialize fields */ };
/// let theme = /* create a theme instance */;
/// let template = fill_license_forms(&mut details, &theme)?;
/// println!("{:?}", template);
/// ```
pub(crate) fn fill_license_forms(
    details: &mut LicenseDetails,
    theme: &dyn Theme,
) -> LictoolResult<Template> {
    let mut template = Template::default();
    let gitconfig = GitConfig::load();
    if details.has_owner() {
        let owner: String = Input::with_theme(theme)
            .with_prompt("Please enter the author's name")
            .show_default(true)
            .default(gitconfig.username)
            .interact_text()
            .unwrap();
        template.owner = Some(owner);
    }
    if details.has_year() {
        let year: i32 = Input::with_theme(theme)
            .with_prompt("Please enter the year of creation")
            .show_default(true)
            .default(Local::now().year())
            .interact_text()
            .unwrap();
        template.year = if year == 0 { None } else { Some(year) };
    }
    if details.has_repo() {
        let repo: String = Input::with_theme(theme)
            .with_prompt("Please enter the program's name")
            .allow_empty(true)
            .interact_text()
            .unwrap();
        template.repo = if repo.is_empty() { None } else { Some(repo) };
    }
    if details.has_email() {
        let email: String = Input::with_theme(theme)
            .with_prompt("Please enter the email")
            .default(gitconfig.email)
            .allow_empty(true)
            .interact_text()
            .unwrap();
        template.email = if email.is_empty() { None } else { Some(email) };
    }
    template.license_text = take(&mut details.license_text);
    Ok(template)
}

/// Writes the rendered license template to a file.
///
/// This function takes a file path and a mutable reference to a
/// `Template`, renders the template as a string, and writes it
/// to the specified file path.
///
/// # Arguments
///
/// * `path` - A reference to a type that can be converted to a
///   `Path`, representing the file path.
/// * `template` - A mutable reference to a `Template` struct to be
///   rendered and written.
///
/// # Returns
///
/// A `Result` indicating the success or failure of the file write
/// operation.
///
/// # Example
///
/// ```
/// let mut template = Template { /* initialize fields */ };
/// write_template("output.txt", &mut template)?;
/// ```
pub(crate) fn write_template<P: AsRef<Path> + Display>(
    path: P,
    template: &mut Template,
) -> Result<(), anyhow::Error> {
    let path_ref = path.as_ref();

    if path_ref.exists() && path_ref.is_file() {
        return Err(Error::AlreadyExists {
            file: path_ref.to_string_lossy().into_owned(),
        }
        .into());
    } else {
        fs::write(&path, template.render())?;
        cprintln!("<green>✔</> <bold>Successfully created {} file.</>", path);
        Ok(())
    }
}

pub(crate) fn interact_write_template<P: AsRef<Path> + Display>(
    path: P,
    template: &mut Template,
) -> Result<(), anyhow::Error> {
    let mut path = path.as_ref().to_string_lossy().into_owned();
    loop {
        match write_template(&path, template) {
            Ok(_) => return Ok(()),
            Err(e) => {
                if matches!(
                    e.downcast_ref::<Error>(),
                    Some(Error::AlreadyExists {
                        file: _
                    })
                ) {
                    cprintln!("<y, bold>\u{f421}</> <bold>{}</>", e.to_string());
                    let new_path: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Please specify a new file name to avoid overwriting.")
                        .default(path.clone())
                        .interact_text()
                        .unwrap();
                    path = new_path;
                } else {
                    return Err(anyhow!("An unknown error occurred: {}", e));
                }
            }
        }
    }
}
