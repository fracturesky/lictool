use color_print::cformat;
use snafu::prelude::*;
pub type LictoolResult<T> = anyhow::Result<T>;

#[derive(Snafu, Debug)]
pub(crate) enum Error {
    #[snafu(display("No license found matching the ID provided."))]
    NotFound,
    #[snafu(display("The {file} file already exists."))]
    AlreadyExists { file: String },
}

pub(crate) fn display_error(err: &anyhow::Error) {
    eprintln!("{}", cformat!("<red, bold>Error:</> {}", err));
    for cause in err.chain().skip(1) {
        eprintln!("{}", cformat!("\n<bold>Caused by:</>"));
        for line in cause.to_string().lines() {
            if line.is_empty() {
                eprintln!();
            } else {
                eprintln!("   {}", line)
            }
        }
    }
}
