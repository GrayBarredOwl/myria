pub use crate::obj::{Object, RustFunc};
pub use crate::r#gen::MyriaRes;

pub type FuncInfo = (String, RustFunc);

pub type MyriaFn = fn(Vec<Object>) -> MyriaRes;
pub type LoadFn = fn() -> Vec<FuncInfo>;
pub type NameFn = fn() -> String;
