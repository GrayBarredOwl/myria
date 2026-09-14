use std::{env, path::PathBuf};

use crate::{myria_settings::lib_load::Library};
use myria::obj::RustFunc;

#[derive(Debug, Default)]
pub struct MyriaConfig {
    file: Option<PathBuf>,
    dylibs: Vec<LibInfo>,
    open_dylibs: Vec<Library>,
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
    pub fn load_libs_to_rsc(&mut self) {
        use myria::stdlib::{init_dyn_funcs, register_function};

        init_dyn_funcs();
        for lib_path in &self.dylibs {
            let lib = Library::new(lib_path.as_str(), lib_load::RTLD_NOW)
                .expect("Could not load library");

            // load function pointer to grab exported functions
            let loader = match lib.get_sym("load") {
                Some(ptr) => ptr,
                None => {
                    eprintln!("Library '{lib_path}' does not contain 'load' function");
                    continue;
                }
            };
            let loader: fn() -> Vec<(String, RustFunc)> =
                unsafe { std::mem::transmute(loader.as_ptr()) };
            
            let plugin_name = match lib.get_sym("name") {
                Some(ptr) => ptr,
                None => {
                    eprintln!("Library '{lib_path}' does not contain 'name' function");
                    continue;
                }
            };
            let plugin_name: fn() -> String =
                unsafe { std::mem::transmute(plugin_name.as_ptr()) }; 
            let plugin_name = plugin_name();


            for (name, rf) in loader() {
                register_function(format!("{plugin_name}.{name}"), rf);
            }

            self.open_dylibs.push(lib);
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
                Some(Library { handle: unsafe { NonNull::new_unchecked(handle) } })
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
