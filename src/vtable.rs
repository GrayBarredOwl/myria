// use crate::ast::{Expression, Scope};
use crate::gen::Operator;
use crate::obj::{Object, Primitive};
use core::fmt;

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
    panic!("Invalid binary operation!");
}
fn invalid_un(_: Primitive) -> Object {
    panic!("Invalid unary operation!");
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
            Object::make_bool(x $op y)
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
                match y {
                    P::Bool(val) => Object::make_int(x $op val as i64),
                    P::Int(val) => Object::make_int(x $op val),
                    P::Float(val) => Object::make_float(x as f64 $op val),
                    _ => panic!("Invalid operation"),
                }
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
        Object::make_int(-val)
    }
}

pub mod bool_vtable {
    use super::*;
    type P = Primitive;

    macro_rules! make_arith_op {
        ($name:ident, $op:tt) => {
            fn $name(x: Primitive, y: Primitive) -> Object {
                let x = get_prim!(x, Bool);
                match y {
                    P::Bool(val) => Object::make_bool((x as i32 $op val as i32) != 0),
                    P::Int(val) => Object::make_int(x as i64 $op val),
                    P::Float(val) => Object::make_float(x as i32 as f64 $op val),
                    _ => panic!("Invalid operation"),
                }
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
    use super::*;

    pub static VTABLE: VTable = VTable {
        eq,
        ..VTable::all_invalid()
    };

    fn eq(x: Primitive, y: Primitive) -> Object {
        Object::make_bool(x == Primitive::Null && y == Primitive::Null)
    }
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
                Object::make_float(result)
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
        Object::make_bool(result)
    }

    fn negt(x: Primitive) -> Object {
        let x = get_prim!(x, Float);
        Object::make_float(-x)
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
    use crate::obj::{List, PrimType};

    use super::*;

    pub static VTABLE: VTable = VTable {
        add,
        eq,
        ..VTable::all_invalid()
    };

    fn add(x: Primitive, y: Primitive) -> Object {
        let x = get_prim!(x, List);
        match y {
            Primitive::List(lst) => {
                Object::make_list(x.elems.into_iter().chain(lst.elems).collect())
            }
            other => {
                let mut elems = x.elems;
                elems.push(Object::new(other));
                Object::make_list(elems)
            }
        }
    }
    fn eq(x: Primitive, y: Primitive) -> Object {
        let x = get_prim!(x, List);
        let y = get_prim!(y, List);
        Object::make_bool(x == y)
    }

    pub fn index(x: List, i: Object) -> Object {
        if i.get_type() != PrimType::Int {
            panic!("Can")
        }
        let i = match i.primitive {
            Primitive::Int(val) => val,
            _ => unreachable!(),
        };

        if i < 0 {
            panic!("Can not index with a negative number! {i}, {x:?}");
        }
        let i = i as usize;
        if i >= x.elems.len() {
            panic!("Index out of bounds! {i}, {}", Object::make_list(x.elems));
        }
        x.elems[i].clone()
    }
}

pub mod function_vtable {
    use super::VTable;

    pub static VTABLE: VTable = VTable::all_invalid();
}

pub mod instance_vtable {
    use super::*;

    pub static VTABLE: VTable = VTable::all_invalid();
}

pub mod type_vtable {
    use super::*;

    pub static VTABLE: VTable = VTable::all_invalid();
}
