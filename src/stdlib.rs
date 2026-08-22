use crate::obj::{Object, PrimType, Primitive, RustFunc};

pub const FUNCS: &[(&str, RustFunc)] = &[
    ("print", RustFunc::new(None, print)),
    ("dbg_print", RustFunc::new(None, dbg_print)),
    ("nop", RustFunc::new(None, nop)),
    ("exit", RustFunc::new(None, exit)),
    ("import", RustFunc::new(Some(1), import)),
    ("mod", RustFunc::new(Some(2), modulus)),
    ("empty", RustFunc::new(Some(0), empty)),
];

fn exit(args: Vec<Object>) -> Object {
    let mut args = args;
    let ret_val = {
        if args.is_empty() {
            0
        } else {
            let o = args.swap_remove(0);
            match PrimType::int_cast(o).primitive {
                Primitive::Int(i) => i as i32,
                Primitive::Null => -1,
                _ => unreachable!(),
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
    use crate::gen;
    use crate::obj::Primitive;
    use std::io::ErrorKind;

    let [fp] = &args[..] else {
        unreachable!();
    };
    if !is_string(&fp.primitive) {
        panic!("import must take a string");
    }
    let Primitive::List(fp) = &fp.primitive else {
        unreachable!();
    };

    if fp.elems.is_empty() {
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
        ErrorKind::NotFound => fp.push_str(gen::EXTENSION),
        _ => panic!("Import error: {err}"),
    }

    match gen::run_file(&fp) {
        Ok(obj) => obj,
        Err(e) => panic!("Import error: {e}"),
    }
}

fn modulus(args: Vec<Object>) -> Object {
    if args.len() != 2 {
        panic!("mod function takes 2 arguments, not {}", args.len());
    }
    type P = Primitive;
    match args[0].primitive {
        P::Int(x) => match args[1].primitive {
            P::Int(y) => Object::make_int(x % y),
            P::Float(y) => Object::make_float(x as f64 % y),
            _ => panic!(
                "mod function only takes ints and floats, not {:?}",
                args[1].get_type()
            ),
        },
        P::Float(x) => match args[1].primitive {
            P::Int(y) => Object::make_float(x % y as f64),
            P::Float(y) => Object::make_float(x % y),
            _ => panic!(
                "mod function only takes ints and floats, not {:?}",
                args[1].get_type()
            ),
        },
        _ => panic!(
            "mod function only takes ints and floats, not {:?}",
            args[0].get_type()
        ),
    }
}
fn empty(args: Vec<Object>) -> Object {
    assert!(args.len() == 0);
    Object::make_inst()
}

fn is_string(p: &Primitive) -> bool {
    match p {
        Primitive::List(l) => l.ltype == Some(PrimType::Char),
        _ => false,
    }
}
