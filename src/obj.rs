use crate::vtable::{self, VTable};
use crate::ast::Expression;


#[derive(Debug, Default, PartialEq, Clone)]
pub enum Primitive {
    #[default]
    Null,
    Int(i64),
    Float(f64),
    Char(char),
    Bool(bool),

    Type(PrimType),
    List(List),
    Function(Function),
    Class(Vec<Primitive>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PrimType {
    Null,
    Bool,
    Int,
    Float,
    Char,

    Type,
    List,
    Function,
    Class,
}

impl PrimType {
    pub fn cast(self, obj: Object) -> Object {
        match self {
            Self::Null => Object::make_null(),
            Self::Bool => Self::bool_cast(obj),
            Self::Int => Self::int_cast(obj),
            Self::Float => Self::float_cast(obj),
            Self::Char => Self::char_cast(obj),

            Self::Type => Self::type_cast(obj),
            Self::List => Self::list_cast(obj),
            Self::Function => Self::func_cast(obj),
            Self::Class => Self::class_cast(obj),
        }
    }
    pub fn bool_cast(obj: Object) -> Object {
        type P = Primitive;
        let b = match obj.primitive {
            P::Null | P::Type(_) => return Object::make_null(),
            other => other.is_truthy(),
        };
        Object::make_bool(b)
    }
    pub fn int_cast(obj: Object) -> Object {
        type P = Primitive;
        let i = match obj.primitive {
            P::Null | P::Type(_) => return Object::make_null(),
            P::Bool(b) => b as i64,
            P::Int(i) => i,
            P::Float(f) => f as i64,
            P::Char(c) => c as i64,
            P::List(List {
                ltype: _,
                ref elems,
            }) => elems.len() as i64,
            P::Function(f) => match f {
                Function::RustFn(RustFunc { num_args, .. }) => num_args.unwrap_or(-1),
                Function::LangFn(LangFunc {
                    ref params,
                    is_variadic: _,
                    body: _,
                }) => params.len() as i64,
            },
            P::Class(ref mems) => mems.len() as i64,
        };
        Object::make_int(i)
    }
    pub fn float_cast(obj: Object) -> Object {
        type P = Primitive;
        let f = match obj.primitive {
            P::Null | P::Char(_) | P::Type(_) | P::List(_) | P::Function(_) | P::Class(_) => {
                return Object::make_null()
            }
            P::Bool(b) => {
                if b {
                    1.0
                } else {
                    0.0
                }
            }
            P::Int(i) => i as f64,
            P::Float(f) => f,
        };
        Object::make_float(f)
    }
    pub fn char_cast(obj: Object) -> Object {
        type P = Primitive;
        let c = match obj.primitive {
            P::Null | P::Float(_) | P::Class(_) | P::Function(_) | P::Type(_) | P::List(_) => {
                return Object::make_null()
            }
            P::Bool(b) => {
                if b {
                    'T'
                } else {
                    'F'
                }
            }
            P::Int(i) => match char::from_u32(i as u32) {
                Some(c) => c,
                None => return Object::make_null(),
            },
            P::Char(c) => c,
        };
        Object::make_char(c)
    }
    pub fn type_cast(obj: Object) -> Object {
        Object::make_type(obj.get_type())
    }
    pub fn list_cast(obj: Object) -> Object {
        Object::make_list(vec![obj.clone()])
    }
    pub fn func_cast(obj: Object) -> Object {
        type P = Primitive;
        match obj.primitive {
            P::Function(f) => Object::make_func_dir(f),
            _ => Object::make_null(),
        }
    }
    pub fn class_cast(obj: Object) -> Object {
        // type P = Primitive;
        match obj.primitive {
            _ => unimplemented!(),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct List {
    pub ltype: Option<PrimType>,
    pub elems: Vec<Object>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Function {
    RustFn(RustFunc),
    LangFn(LangFunc),
}

#[derive(Clone)]
pub struct Object {
    pub primitive: Primitive,
    pub vtable: &'static VTable,
}
impl Default for Object {
    fn default() -> Self {
        Self::make_null()
    }
}
impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Obj({:?})", self.primitive)
    }
}

impl Object {
    pub fn make_int(n: i64) -> Self {
        Self {
            primitive: Primitive::Int(n),
            vtable: &vtable::int_vtable::VTABLE,
        }
    }
    pub fn make_float(n: f64) -> Self {
        Self {
            primitive: Primitive::Float(n),
            vtable: &vtable::float_vtable::VTABLE,
        }
    }
    pub fn make_bool(b: bool) -> Self {
        Self {
            primitive: Primitive::Bool(b),
            vtable: &vtable::bool_vtable::VTABLE,
        }
    }
    pub fn make_char(c: char) -> Self {
        Self {
            primitive: Primitive::Char(c),
            vtable: &vtable::char_vtable::VTABLE,
        }
    }
    pub fn make_null() -> Self {
        Self {
            primitive: Primitive::Null,
            vtable: &vtable::null_vtable::VTABLE,
        }
    }
    pub fn make_func(params: Vec<String>, variadic: bool, body: Box<Expression>) -> Self {
        Self {
            primitive: Primitive::Function(Function::LangFn(LangFunc::new(params, variadic, body))),
            vtable: &vtable::function_vtable::VTABLE,
        }
    }
    pub fn make_func_dir(func: Function) -> Self {
        Self {
            primitive: Primitive::Function(func),
            vtable: &vtable::function_vtable::VTABLE,
        }
    }
    pub fn make_rust_func(func: RustFunc) -> Self {
        Self {
            primitive: Primitive::Function(Function::RustFn(func)),
            vtable: &vtable::function_vtable::VTABLE,
        }
    }
    pub fn make_list(vec: Vec<Object>) -> Self {
        if vec.is_empty() {
            return Self {
                primitive: Primitive::List(List {
                    ltype: None,
                    elems: vec![],
                }),
                vtable: &vtable::list_vtable::VTABLE,
            };
        }
        // let mut is_uniform = true;
        let obj_type = vec.first().unwrap().get_type();
        let is_uniform = vec.iter().skip(1).all(|o| o.get_type() == obj_type);

        Self {
            primitive: Primitive::List(List {
                ltype: if is_uniform {
                    Some(vec[0].get_type())
                } else {
                    None
                },
                elems: vec,
            }),
            vtable: &vtable::list_vtable::VTABLE,
        }
    }
    pub fn make_type(t: PrimType) -> Self {
        Self {
            primitive: Primitive::Type(t),
            vtable: &vtable::type_vtable::VTABLE,
        }
    }
    pub fn make_expr(self) -> Expression {
        Expression::Value(self)
    }
    pub fn get_type(&self) -> PrimType {
        type P = Primitive;
        type PT = PrimType;
        match self.primitive {
            P::Int(_) => PT::Int,
            P::Float(_) => PT::Float,
            P::Char(_) => PT::Char,
            P::Bool(_) => PT::Bool,
            P::Null => PT::Null,
            P::Type(_) => PT::Type,

            P::List(_) => PT::List,
            P::Function(_) => PT::Function,
            P::Class(_) => PT::Class,
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.primitive == other.primitive
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        type P = Primitive;
        write!(
            f,
            "{}",
            match self.primitive {
                P::Int(v) => v.to_string(),
                P::Float(v) => v.to_string(),
                P::Bool(v) => v.to_string(),
                P::Char(v) => v.to_string(),
                P::Class(_) => String::from("Class"),
                P::Type(v) => format!("{v:?}"),
                P::Function(_) => String::from("Function"),
                P::Null => String::from("Null"),
                P::List(List {
                    ltype: Some(PrimType::Char),
                    ref elems,
                }) => elems
                    .iter()
                    .map(|c| match c.primitive {
                        Primitive::Char(c) => c,
                        _ => unreachable!(),
                    })
                    .collect(),
                P::List(List {
                    ltype: _,
                    ref elems,
                }) => format!(
                    "[{}]",
                    elems
                        .iter()
                        .map(ToString::to_string)
                        .reduce(|a, o| format!("{a}, {o}"))
                        .unwrap_or_default()
                ),
            }
        )
    }
}
impl Primitive {
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Null => false,
            Self::Int(val) => *val != 0,
            Self::Float(val) => *val != 0.0,
            Self::Char(val) => *val != '\0',
            Self::Bool(val) => *val,

            Self::Type(_) => true,
            Self::List(List { ltype: _, elems }) => !elems.is_empty(),
            Self::Function(_) => true,
            Self::Class(_) => true,
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(unpredictable_function_pointer_comparisons)]
pub struct RustFunc {
    pub num_args: Option<i64>,
    pub fn_ptr: fn(Vec<Object>) -> Object,
}
impl RustFunc {
    pub const fn new(num_args: Option<i64>, fn_ptr: fn(Vec<Object>) -> Object) -> Self {
        Self { num_args, fn_ptr }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct LangFunc {
    pub params: Vec<String>,
    pub is_variadic: bool,
    pub body: Box<Expression>,
}
impl LangFunc {
    fn new(params: Vec<String>, is_variadic: bool, body: Box<Expression>) -> Self {
        Self { params, is_variadic, body }
    }
}