use std::{
    env,
    path::{Path, PathBuf},
};

use crate::ast::Scope;

#[derive(Debug, Default)]
pub struct MyriaConfig {
    file: Option<PathBuf>,
    dylibs: Vec<LibInfo>,
    // future flags
}
impl MyriaConfig {
    pub fn from_args(args: env::Args) -> Self {
        let mut args = args;
        let mut config = MyriaConfig::default();

        let _this_path = args.next().expect("Always contains it's own path");
        while let Some(next_arg) = args.next() {
            config.process_arg(&next_arg, &mut args);
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
    pub fn load_libs_to_rsc(&self) {
        use crate::stdlib::{init_dyn_funcs, register_function};

        init_dyn_funcs();;
        for lib_path in &self.dylibs {
            let lib_path = lib_path.as_path();
            todo!()
            // register_function(name, func);
        } 
    }
    fn process_arg(&mut self, cur_arg: &str, rem_args: &mut std::env::Args) {
        match cur_arg {
            "--lib" => {
                self.dylibs.push(
                    rem_args
                        .next()
                        .expect("Library path should follow --lib flag")
                        .into(),
                );
            }
            file => {
                if self.file.is_none() {
                    self.file = Some(file.into());
                } else {
                    panic!("Can not run multiple files at once: Only enter 1 file to execute");
                }
            }
        }
    }
}
type LibInfo = PathBuf;
// #[derive(Debug, Clone)]
// struct LibInfo {
// path: PathBuf,
// name: Option<String>,
// }
