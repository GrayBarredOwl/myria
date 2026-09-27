use myria::plugin::*;



#[unsafe(no_mangle)]
pub fn load() -> Vec<FuncInfo> {
    vec![("first".into(), RustFunc::new(Some(2), first))]
}
#[unsafe(no_mangle)]
pub fn name() -> String {
    "sample-plugin".into()
}

fn first(args: Vec<Object>) -> MyriaRes {
    Ok(args[0].clone())
}