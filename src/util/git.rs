use git2::Config;

#[derive(Debug, Default)]
pub(crate) struct GitConfig {
    pub(crate) username: String,
    pub(crate) email: String,
}

impl GitConfig {
    pub fn load() -> Self {
        retrieve_git_config().unwrap_or_default()
    }
}

fn retrieve_git_config() -> Result<GitConfig, git2::Error> {
    let config = Config::open_default()?;
    Ok(GitConfig {
        username: config.get_string("user.name").unwrap_or_default(),
        email: config.get_string("user.email").unwrap_or_default(),
    })
}
