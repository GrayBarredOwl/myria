use std::{env, path::PathBuf};

#[derive(Debug, Default)]
pub struct Config {
    pub file: Option<PathBuf>,
    // future flags
}
pub fn parse_args(args: env::Args) -> Config {
    let mut args = args;
    let mut config = Config::default();

    let _this_path = args.next().expect("Always contains it's own path");
    if let Some(file_path) = args.next() {
        config.file = Some(file_path.into());
    }
    config
}
