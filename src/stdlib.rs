use crate::{
    gen::{MyriaErr, MyriaRes},
    obj::{List, Object, PrimType, Primitive, RustFunc},
};

pub static FUNCS: &[(&str, RustFunc)] = &[
    // IO
    ("print", RustFunc::new(None, io::print)),
    ("dbg_print", RustFunc::new(None, io::dbg_print)),
    ("file", RustFunc::new(Some(1), io::file)),
    // System
    ("exit", RustFunc::new(None, system::exit)),
    ("rsc", RustFunc::new(None, system::rust_call)),
    ("eval", RustFunc::new(Some(1), system::eval)),
    ("import", RustFunc::new(Some(1), system::import)),
    // General purpose
    ("mod", RustFunc::new(Some(2), general::modulus)),
    ("empty", RustFunc::new(Some(0), general::empty)),
    ("str", RustFunc::new(Some(1), general::to_string)),
    ("get", RustFunc::new(Some(2), general::get)),
    ("replace", RustFunc::new(Some(3), general::replace)),
];

use std::{collections::HashMap, sync::OnceLock};
static mut DY_FUNCS: OnceLock<HashMap<String, RustFunc>> = OnceLock::new();
pub fn init_dyn_funcs() {
    unsafe { DY_FUNCS.set(HashMap::new()) };
}
pub fn dynamic_functions() -> &'static HashMap<String, RustFunc> {
    unsafe {
        DY_FUNCS.get_or_init(|| {
            panic!("dynamic functions uninitializied");
        })
    }
}
unsafe fn get_dyn_fns_mut() -> &'static mut HashMap<String, RustFunc> {
    unsafe { DY_FUNCS.get_mut().unwrap() }
}

pub fn register_function(name: String, func: RustFunc) {
    let funcs = unsafe { get_dyn_fns_mut() };
    funcs.insert(name, func).expect("Function already exists");
}
mod io {
    use super::*;
    pub fn dbg_print(args: Vec<Object>) -> MyriaRes {
        println!(
            "{}",
            args.iter()
                .map(|o| format!("{o:?}"))
                .reduce(|a, val| format!("{a}, {val}"))
                .unwrap_or_default()
        );
        Ok(Object::default())
    }
    pub fn print(args: Vec<Object>) -> MyriaRes {
        println!(
            "{}",
            args.iter()
                .map(ToString::to_string)
                .reduce(|a, val| format!("{a} {val}"))
                .unwrap_or_default()
        );
        Ok(Object::default())
    }
    pub fn file(args: Vec<Object>) -> MyriaRes {
        let [fp] = &args[..] else {
            unreachable!();
        };
        if !fp.primitive.is_string() {
            return Err(MyriaErr::InvalidOperation(
                "file function expects a string".into(),
            ));
        }
        let Primitive::List(fp) = &fp.primitive else {
            unreachable!();
        };
        let fp = fp
            .elems
            .iter()
            .map(|c| match c.primitive {
                Primitive::Char(c) => c,
                _ => unreachable!(),
            })
            .collect::<String>();

        let string = std::fs::read_to_string(fp).map_err(|e| MyriaErr::FileError(e.to_string()))?;
        Ok(Object::make_str(&string))
    }
}

mod system {
    use super::*;
    use crate::gen;

    pub fn exit(args: Vec<Object>) -> MyriaRes {
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
    pub fn eval(args: Vec<Object>) -> MyriaRes {
        use crate::obj::List;
        let program_str = &args[0];
        let Primitive::List(List {
            ltype: Some(PrimType::Char),
            ref elems,
        }) = program_str.primitive
        else {
            return Err(MyriaErr::InvalidType(program_str.get_type()));
        };
        let program = elems
            .iter()
            .map(|c| match c.primitive {
                Primitive::Char(c) => c,
                _ => unreachable!(),
            })
            .collect::<String>();

        gen::run_myria(&program)
    }

    pub fn import(args: Vec<Object>) -> MyriaRes {
        use crate::gen;
        use crate::obj::Primitive;

        let [fp] = &args[..] else {
            unreachable!();
        };
        if !fp.primitive.is_string() {
            return Err(MyriaErr::InvalidOperation(
                "import must take a string".into(),
            ));
        }
        let Primitive::List(fp) = &fp.primitive else {
            unreachable!();
        };

        if fp.elems.is_empty() {
            return Err(MyriaErr::InvalidOperation(
                "import must take a non-zero length string".into(),
            ));
        }

        let mut fp = fp
            .elems
            .iter()
            .map(|c| match c.primitive {
                Primitive::Char(c) => c,
                _ => unreachable!(),
            })
            .collect::<String>();
        fp.push_str(crate::gen::EXTENSION);

        gen::run_file(&fp)
    }

    pub fn rust_call(args: Vec<Object>) -> MyriaRes {
        use crate::ast::function_call_rust;

        let name = args.get(0)
            .ok_or(MyriaErr::InvalidOperation("rsc must have at least 1 argument".into()))?;
        if !name.primitive.is_string() {
            return Err(MyriaErr::InvalidOperation(
                "rsc must be called with a string as the first argument".into(),
            ));
        }
        let name = match &name.primitive {
            Primitive::List(List {
                ltype: Some(PrimType::Char),
                ref elems,
            }) => elems,
            _ => unreachable!(),
        }
        .iter()
        .map(|c| match c.primitive {
            Primitive::Char(c) => c,
            _ => unreachable!(),
        })
        .collect::<String>();

        let rf = match dynamic_functions().get(&name) {
            Some(rf) => *rf,
            None => return Err(MyriaErr::VariableDNE(name)),
        };

        let fn_args = args.into_iter().skip(1).collect();

        function_call_rust(rf, fn_args)
    }
}

mod general {
    use super::*;
    pub fn modulus(args: Vec<Object>) -> MyriaRes {
        if args.len() != 2 {
            panic!("mod function takes 2 arguments, not {}", args.len());
        }
        type P = Primitive;
        match args[0].primitive {
            P::Int(x) => match args[1].primitive {
                P::Int(y) => Ok(Object::make_int(x % y)),
                P::Float(y) => Ok(Object::make_float(x as f64 % y)),
                _ => Err(MyriaErr::InvalidOperation(format!(
                    "mod function only takes ints and floats, not {:?}",
                    args[1].get_type()
                ))),
            },
            P::Float(x) => match args[1].primitive {
                P::Int(y) => Ok(Object::make_float(x % y as f64)),
                P::Float(y) => Ok(Object::make_float(x % y)),
                _ => Err(MyriaErr::InvalidOperation(format!(
                    "mod function only takes ints and floats, not {:?}",
                    args[1].get_type()
                ))),
            },
            _ => Err(MyriaErr::InvalidOperation(format!(
                "mod function only takes ints and floats, not {:?}",
                args[0].get_type()
            ))),
        }
    }
    pub fn empty(args: Vec<Object>) -> MyriaRes {
        assert!(args.is_empty());
        Ok(Object::make_inst())
    }

    pub fn to_string(args: Vec<Object>) -> MyriaRes {
        assert!(args.len() == 1);
        Ok(Object::make_str(&args[0].to_string()))
    }
    pub fn get(args: Vec<Object>) -> MyriaRes {
        assert!(args.len() == 2);
        if args[0].get_type() != PrimType::List {
            Err(MyriaErr::InvalidOperation("Can not index non-List".into()))
        } else if args[1].get_type() != PrimType::Int {
            Err(MyriaErr::InvalidOperation("Index must be an Int".into()))
        } else {
            let Primitive::List(l) = &args[0].primitive else {
                unreachable!();
            };
            let Primitive::Int(mut index) = args[1].primitive else {
                unreachable!();
            };

            if index >= l.elems.len() as i64 {
                return Err(MyriaErr::OutOfBounds(index));
            }
            if index < 0 {
                index += l.elems.len() as i64;
            }

            if index < 0 {
                Err(MyriaErr::OutOfBounds(index - l.elems.len() as i64))
            } else {
                Ok(l.elems[index as usize].clone())
            }
        }
    }
    pub fn replace(args: Vec<Object>) -> MyriaRes {
        assert!(args.len() == 3);
        if args[0].get_type() != PrimType::List {
            Err(MyriaErr::InvalidOperation("Can not index non-List".into()))
        } else if args[1].get_type() != PrimType::Int {
            Err(MyriaErr::InvalidOperation("Index must be an Int".into()))
        } else {
            let Primitive::List(l) = &args[0].primitive else {
                unreachable!();
            };
            let Primitive::Int(mut index) = args[1].primitive else {
                unreachable!();
            };

            if index >= l.elems.len() as i64 {
                return Err(MyriaErr::OutOfBounds(index));
            }
            if index < 0 {
                index += l.elems.len() as i64;
            }

            if index < 0 {
                Err(MyriaErr::OutOfBounds(index - l.elems.len() as i64))
            } else {
                let first_half = l.elems.iter().take(index as usize);
                let second_half = l.elems.iter().skip(1 + index as usize);
                Ok(Object::make_list(
                    first_half
                        .chain(std::iter::once(&args[2]))
                        .chain(second_half)
                        .cloned()
                        .collect(),
                ))
            }
        }
    }
}
