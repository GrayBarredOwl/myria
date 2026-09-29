use std::{env, path::PathBuf};
use myria::util;
use crate::myria_settings::lib_load::Library;

#[derive(Debug)]
pub struct MyriaConfig {
    file: Option<PathBuf>,
    dylibs: Vec<LibInfo>,
    open_dylibs: Vec<Library>,
    version_flag: bool,
    pub debug_print_flag: bool,
    // future flags
}
impl Default for MyriaConfig {
    fn default() -> Self {
        MyriaConfig {
            file: None,
            dylibs: vec![String::from("myrialib/target/debug/libmyrialib.dylib")],
            open_dylibs: vec![],
            version_flag: false,
            debug_print_flag: false,
        }
    }
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
    pub fn docs_not_execute(&self) -> bool {
        self.version_flag
    }
    pub fn program_string(&self) -> Option<String> {
        self.file
            .as_ref()
            .map(|fp| match std::fs::read_to_string(fp) {
                Ok(s) => s,
                Err(err) => panic!("Couldn't read file({}): {err}", fp.display()),
            })
    }
    pub fn display_requested_info(&self) {
        println!("Myria version: {}", env!("CARGO_PKG_VERSION"));
    }
    pub fn execute_setup(&mut self) {
        if self.docs_not_execute() {
            self.display_requested_info();
            std::process::exit(0);
        }
        if self.debug_print_flag {
            util::set_debug_print(true);
        }

        self.load_libs_to_rsc();
    }
    pub fn load_libs_to_rsc(&mut self) {
        use myria::plugin::{LoadFn, NameFn};
        use myria::stdlib::{init_dyn_funcs, register_function};

        init_dyn_funcs();
        for lib_path in &self.dylibs {
            let Some(lib) = Library::new(lib_path.as_str(), lib_load::RTLD_NOW) else {
                eprintln!("Library '{lib_path}' could not be opened");
                continue;
            };

            // load function pointer to grab exported functions
            let Some(loader) = lib.get_sym("load") else {
                eprintln!("Library '{lib_path}' does not contain 'load' function");
                continue;
            };
            let loader: LoadFn = unsafe { std::mem::transmute(loader.as_ptr()) };

            let Some(plugin_name) = lib.get_sym("name") else {
                eprintln!("Library '{lib_path}' does not contain 'name' function");
                continue;
            };
            let plugin_name: NameFn = unsafe { std::mem::transmute(plugin_name.as_ptr()) };
            let plugin_name = plugin_name();

            for (name, rf) in loader() {
                register_function(format!("{plugin_name}.{name}"), rf);
            }

            self.open_dylibs.push(lib);
        }
    }

    fn process_arg(&mut self, cur_arg: &str, rem_args: &mut std::env::Args) {
        if cur_arg.starts_with("-") && cur_arg.chars().nth(1) != Some('-') {
            for c in cur_arg.chars().skip(1) {
                match c {
                    'v' => self.version_flag = true,
                    'd' => self.debug_print_flag = true,
                    _ => (),
                }
            }
        } else {
            match cur_arg {
                "--lib" => {
                    self.dylibs.push(
                        rem_args
                            .next()
                            .expect("Library path should follow --lib flag"),
                    );
                }
                "--debug" => {
                    self.debug_print_flag = true;
                }
                "--version" => {
                    self.version_flag = true;
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
}
type LibInfo = String;
// #[derive(Debug, Clone)]
// struct LibInfo {
// path: PathBuf,
// name: Option<String>,
// }

#[cfg(target_os = "macos")]
mod lib_load {
    use std::{
        ffi::{c_char, c_int, c_void, CString},
        ptr::NonNull,
    };
    extern "C" {
        fn dlopen(path: *const c_char, mode: c_int) -> *mut c_void;
        fn dlclose(handle: *mut c_void) -> c_int;
        fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
        fn dlerror() -> *const c_char;
        fn puts(string: *const c_char) -> c_int; // should use with stderr, future feature
    }

    #[cfg(target_os = "macos")]
    #[allow(unused)]
    pub const RTLD_LAZY: c_int = 1;
    #[cfg(target_os = "macos")]
    pub const RTLD_NOW: c_int = 2;

    #[derive(Debug)]
    pub struct Library {
        handle: NonNull<c_void>,
    }

    impl Library {
        pub fn new(path: &str, mode: c_int) -> Option<Self> {
            let cpath = CString::new(path).ok()?;

            let handle = unsafe { dlopen(cpath.as_ptr(), mode) };
            if handle.is_null() {
                print_dl_err("dlopen err: ");
                None
            } else {
                Some(Library {
                    handle: unsafe { NonNull::new_unchecked(handle) },
                })
            }
        }
        pub fn get_sym(&self, name: &str) -> Option<NonNull<c_void>> {
            let name = CString::new(name).expect("Could not convert &str to CString");
            let sym_ptr = unsafe { dlsym(self.handle.as_ptr(), name.as_ptr()) };
            if sym_ptr.is_null() {
                print_dl_err("dlsym err: ");
                None
            } else {
                Some(unsafe { NonNull::new_unchecked(sym_ptr) })
            }
        }
    }

    impl Drop for Library {
        fn drop(&mut self) {
            unsafe {
                if dlclose(self.handle.as_ptr()) != 0 {
                    print_dl_err("dlclose err: ");
                }
            }
        }
    }

    fn print_dl_err(prefix: &str) {
        let err_msg = unsafe { dlerror() };
        print!("{prefix}");
        unsafe { puts(err_msg) };
    }
}
