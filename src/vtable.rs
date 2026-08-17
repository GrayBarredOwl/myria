use crate::ast::{Object, Primitive};
use crate::gen::Operator;
use core::fmt;

impl Default for Object {
    fn default() -> Self {
        Self {
            primitive: Primitive::Null,
            vtable: &null_vtable::VTABLE,
        }
    }
}

pub type BinOpFn = fn(Primitive, Primitive) -> Object;
pub type UnOpFn = fn(Primitive) -> Object;

#[derive(Clone)]
pub struct VTable {
    pub add: BinOpFn,
    pub mul: BinOpFn,
    pub div: BinOpFn,

    pub eq: BinOpFn,
    pub lt: BinOpFn,

    pub and: BinOpFn,
    pub or: BinOpFn,
    pub not: UnOpFn,

    pub negt: UnOpFn,
}

impl VTable {
    const fn all_invalid() -> Self {
        Self {
            add: invalid_bn,
            mul: invalid_bn,
            div: invalid_bn,

            eq: invalid_bn,
            lt: invalid_bn,

            and: invalid_bn,
            or: invalid_bn,
            not: invalid_un,

            negt: invalid_un,
        }
    }
}

impl fmt::Debug for VTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<VTABLE: {self:p}>")
    }
}

fn invalid_bn(_: Primitive, _: Primitive) -> Object {
    panic!("Invalid operation!");
}
fn invalid_un(_: Primitive) -> Object {
    panic!("Invalid operation!");
}

macro_rules! get_prim {
    ($var:expr, $variant:tt) => {
        match $var {
            Primitive::$variant(val) => val,
            _ => panic!(
                "{} vtable called without {} as arg",
                stringify!($variant),
                stringify!($variant)
            ),
        }
    };
}

pub fn wrap_prim(prim: Primitive) -> Object {
    type P = Primitive;
    let vtable = match prim {
        P::Null => &null_vtable::VTABLE,
        P::Int(_) => &int_vtable::VTABLE,
        P::Float(_) => &float_vtable::VTABLE,
        P::Char(_) => &char_vtable::VTABLE,
        P::Bool(_) => &bool_vtable::VTABLE,
        P::List(_) => &list_vtable::VTABLE,
        P::Function(_) => &function_vtable::VTABLE,
        P::Class(_) => &class_vtable::VTABLE,
    };

    Object {
        primitive: prim,
        vtable,
    }
}

pub fn pick_binfunc(op: Operator, vtable: &VTable) -> BinOpFn {
    type O = Operator;
    match op {
        O::Plus => vtable.add,
        O::Star => vtable.mul,
        O::Slash => vtable.div,

        O::Equals => vtable.eq,
        O::Lt => vtable.lt,

        O::And => vtable.and,
        O::Or => vtable.or,

        _ => panic!("Invalid operator: {op:?} in pick_binfunc"),
    }
}

pub fn pick_unfunc(op: Operator, vtable: &VTable) -> UnOpFn {
    type O = Operator;
    match op {
        O::Not => vtable.not,
        O::Minus => vtable.negt,
        _ => panic!("Invalid operator: {op:?} in pick_unfunc"),
    }
}

macro_rules! make_logic_op {
    ($ty:tt, $name:ident, $op:tt) => {
        fn $name(x: Primitive, y: Primitive) -> Object {
            let x = get_prim!(x, $ty);
            let y = get_prim!(y, $ty);
            wrap_prim(Primitive::Bool(x $op y))
        }
    };
}

pub mod int_vtable {
    use super::*;
    type P = Primitive;

    macro_rules! make_arith_op {
        ($name:ident, $op:tt) => {
            fn $name(x: Primitive, y: Primitive) -> Object {
                let x = get_prim!(x, Int);
                let result = match y {
                    P::Bool(val) => P::Int(x $op val as i64),
                    P::Int(val) => P::Int(x $op val),
                    P::Float(val) => P::Float(x as f64 $op val),
                    _ => panic!("Invalid operation"),
                };
                wrap_prim(result)
            }
        };
    }

    pub static VTABLE: VTable = VTable {
        add,
        mul,
        div,

        eq,
        lt,

        negt,
        ..VTable::all_invalid()
    };

    make_arith_op!(add, +);
    make_arith_op!(mul, *);
    make_arith_op!(div, /);

    make_logic_op!(Int, eq, ==);
    make_logic_op!(Int, lt, <);

    fn negt(int: Primitive) -> Object {
        let Primitive::Int(val) = int else {
            panic!("Int vtable negate called without an int: {int:?}");
        };
        wrap_prim(Primitive::Int(-val))
    }
}

pub mod bool_vtable {
    use super::*;
    type P = Primitive;

    macro_rules! make_arith_op {
        ($name:ident, $op:tt) => {
            fn $name(x: Primitive, y: Primitive) -> Object {
                let x = get_prim!(x, Bool);
                let result = match y {
                    P::Bool(val) => P::Bool((x as i32 $op val as i32) != 0),
                    P::Int(val) => P::Int(x as i64 $op val),
                    P::Float(val) => P::Float(x as i32 as f64 $op val),
                    _ => panic!("Invalid operation"),
                };
                wrap_prim(result)
            }
        };
    }

    pub static VTABLE: VTable = VTable {
        add,
        mul,

        eq,
        lt,

        and,
        or,
        not,
        ..VTable::all_invalid()
    };

    make_arith_op!(add, +);
    make_arith_op!(mul, *);

    make_logic_op!(Bool, eq, ==);
    make_logic_op!(Bool, lt, < );

    fn and(x: Primitive, y: Primitive) -> Object {
        let x = get_prim!(x, Bool);
        let y = get_prim!(y, Bool);
        Object::make_bool(x && y)
    }
    fn or(x: Primitive, y: Primitive) -> Object {
        let x = get_prim!(x, Bool);
        let y = get_prim!(y, Bool);
        Object::make_bool(x || y)
    }
    fn not(x: Primitive) -> Object {
        let x = get_prim!(x, Bool);
        Object::make_bool(!x)
    }
}

pub mod null_vtable {
    use super::VTable;

    pub static VTABLE: VTable = VTable::all_invalid();
}

pub mod float_vtable {
    use super::*;
    type P = Primitive;

    macro_rules! make_arith_op {
        ($name:ident, $op:tt) => {
            fn $name(x: Primitive, y: Primitive) -> Object {
                let x = get_prim!(x, Float);
                let result = match y {
                    P::Bool(val) => x $op val as i32 as f64,
                    P::Int(val) => x $op val as f64,
                    P::Float(val) => x $op val,
                    _ => panic!("Invalid operation"),
                };
                wrap_prim(Primitive::Float(result))
            }
        };
    }

    pub static VTABLE: VTable = VTable {
        add,
        mul,
        div,

        eq,
        lt,

        negt,
        ..VTable::all_invalid()
    };

    make_arith_op!(add, +);
    make_arith_op!(mul, *);
    make_arith_op!(div, /);

    make_logic_op!(Float, lt, < );

    fn eq(x: Primitive, y: Primitive) -> Object {
        let epsilon = 1e-6;

        let x = get_prim!(x, Float);
        let result = match y {
            P::Bool(val) => ((val as i32 as f64) - x).abs() < epsilon,
            P::Int(val) => (val as f64 - x).abs() < epsilon,
            P::Float(val) => (val - x).abs() < epsilon,
            _ => panic!("Invalid!"),
        };
        wrap_prim(Primitive::Bool(result))
    }

    fn negt(x: Primitive) -> Object {
        let x = get_prim!(x, Float);
        wrap_prim(Primitive::Float(-x))
    }
}

pub mod char_vtable {
    use super::*;

    pub static VTABLE: VTable = VTable {
        eq,
        lt,
        ..VTable::all_invalid()
    };

    make_logic_op!(Char, eq, ==);
    make_logic_op!(Char, lt, < );
}

pub mod list_vtable {
    use super::*;

    pub static VTABLE: VTable = VTable {
        add,
        eq,
        ..VTable::all_invalid()
    };

    fn add(x: Primitive, y: Primitive) -> Object {
        let x = get_prim!(x, List);
        let y = get_prim!(y, List);
        Object::make_list(x.elems.into_iter().chain(y.elems.into_iter()).collect())
    }
    fn eq(x: Primitive, y: Primitive) -> Object {
        let x = get_prim!(x, List);
        let y = get_prim!(y, List);
        wrap_prim(Primitive::Bool(x == y))
    } 
}

pub mod function_vtable {
    use super::VTable;

    pub static VTABLE: VTable = VTable::all_invalid();
}

pub mod class_vtable {
    use super::*;

    pub static VTABLE: VTable = VTable::all_invalid();
}
