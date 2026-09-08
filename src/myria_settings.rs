use std::{env, path::PathBuf};

#[derive(Debug, Default)]
pub struct MyriaConfig {
    pub file: Option<PathBuf>,
    // future flags
}
impl MyriaConfig {
    pub fn from_args(args: env::Args) -> Self {
        let mut args = args;
        let mut config = MyriaConfig::default();

        let _this_path = args.next().expect("Always contains it's own path");
        if let Some(file_path) = args.next() {
            config.file = Some(file_path.into());
        }
        config
    }
    pub fn program_string(&self) -> Option<String> {
        self.file
            .as_ref()
            .map(|fp| match std::fs::read_to_string(fp) {
                Ok(s) => s,
                Err(err) => panic!("Couldn't read file({}): {err}", fp.display()),
            })
    }
}
