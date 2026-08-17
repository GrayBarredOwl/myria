use crate::ast::{Object, PrimType, Primitive, RustFunc};

macro_rules! reg_func {
    ($($f:ident),+ $(,)?) => {
        &[ $( (stringify!($f), $f), )* ]
    };
}

pub const FUNCS: &[(&'static str, RustFunc)] = reg_func![print, dbg_print, nop, exit, import];

fn exit(args: Vec<Object>) -> Object {
    use crate::ast::Primitive;
    let ret_val = {
        if args.len() == 0 {
            0
        } else {
            let o = &args[0];
            match o.primitive {
                Primitive::Int(i) => i as i32,
                Primitive::Float(f) => f as i32,
                Primitive::Char(c) => c as i32,
                Primitive::Bool(b) => b as i32,
                Primitive::Null => 0,
                _ => 1,
            }
        }
    };

    std::process::exit(ret_val);
}

fn dbg_print(args: Vec<Object>) -> Object {
    println!(
        "{}",
        args.iter()
            .map(|o| format!("{o:?}"))
            .reduce(|a, val| format!("{a}, {val}"))
            .unwrap_or_default()
    );
    Object::default()
}
fn print(args: Vec<Object>) -> Object {
    println!(
        "{}",
        args.iter()
            .map(ToString::to_string)
            .reduce(|a, val| format!("{a}, {val}"))
            .unwrap_or_default()
    );
    Object::default()
}

fn nop(_args: Vec<Object>) -> Object {
    Object::default()
}

fn import(args: Vec<Object>) -> Object {
    use crate::ast::Primitive;
    use crate::gen;
    use std::io::ErrorKind;

    let [fp] = &args[..] else {
        panic!(
            "import takes exactly 1 argument, not {} ({:?})",
            args.len(),
            args
        );
    };
    if !is_string(&fp.primitive) {
        panic!("import must take a string");
    }
    let Primitive::List(fp) = &fp.primitive else {
        unreachable!();
    };

    if fp.elems.len() == 0 {
        panic!("import must take a non-zero length string");
    }

    let mut fp = fp
        .elems
        .iter()
        .map(|c| match c.primitive {
            Primitive::Char(c) => c,
            _ => unreachable!(),
        })
        .collect::<String>();

    let err = match gen::run_file(&fp) {
        Ok(obj) => return obj,
        Err(e) => e,
    };
    match err.kind() {
        ErrorKind::NotFound => fp.extend(".mylang".chars()),
        _ => panic!("Import error: {err}"),
    }

    match gen::run_file(&fp) {
        Ok(obj) => return obj,
        Err(e) => panic!("Import error: {e}"),
    }
}

fn is_string(p: &Primitive) -> bool {
    match p {
        Primitive::List(l) => l.ltype == Some(PrimType::Char),
        _ => return false,
    }
}
