use crate::ast::{Expression, Object, Scope};

fn print_obj(obj: Object) {
    println!("{obj:?}");
}
pub fn print(args: Vec<Object>) -> Object {
    println!("{}", args.iter()
        .map(ToString::to_string) 
        .reduce(|a, val| format!("{a}, {val}"))
        .unwrap_or_default()
    );

    Object::default()
}

pub fn nop(_args: Vec<Object>) -> Object {
    Object::default()
}