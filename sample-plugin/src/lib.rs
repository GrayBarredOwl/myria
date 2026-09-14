use myria::obj::{Object, RustFunc};
use myria::r#gen::MyriaRes;


#[unsafe(no_mangle)]
pub fn load() -> Vec<(String, RustFunc)> {
    vec![
        ("example".into(), RustFunc::new(None, example)),
    ]
}
#[unsafe(no_mangle)]
pub fn name() -> String {
    "sample_plugin".into()
}

fn example(args: Vec<Object>) -> MyriaRes {
    Ok(args[0].clone())
}