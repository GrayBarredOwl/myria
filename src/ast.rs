use crate::gen::Operator;
use crate::vtable::{self, pick_binfunc, pick_unfunc, VTable};
use std::collections::HashMap;

#[derive(Debug, Default, PartialEq, Clone)]
pub enum Primitive {
    #[default] Null,
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
    fn cast(self, obj: Object) -> Object {
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
    fn bool_cast(obj: Object) -> Object {
        type P = Primitive;
        let b = match obj.primitive {
            P::Null | P::Type(_) => return Object::make_null(),
            other => other.is_truthy(),
        };
        Object::make_bool(b)
    }
    fn int_cast(obj: Object) -> Object {
        type P = Primitive;
        let i = match obj.primitive {
            P::Null | P::Type(_) => return Object::make_null(),
            P::Bool(b) => b as i64,
            P::Int(i) => i,
            P::Float(f) => f as i64,
            P::Char(c) => c as i64,
            P::List(List { ltype: _, ref elems }) => elems.len() as i64,
            P::Function(f) => match f {
                Function::RustFn(RustFunc { num_args, .. }) => num_args.unwrap_or(-1),
                Function::LangFn(ref pnames, _) => pnames.len() as i64,
            },
            P::Class(ref mems) => mems.len() as i64,

        };
        Object::make_int(i)
    }
    fn float_cast(obj: Object) -> Object {
        type P = Primitive;
        let f = match obj.primitive {
            P::Null | P::Char(_) | P::Type(_) | P::List(_) | P::Function(_) | P::Class(_) => return Object::make_null(),
            P::Bool(b) => if b { 1.0 } else { 0.0 },
            P::Int(i) => i as f64,
            P::Float(f) => f,
        };
        Object::make_float(f)
    }
    fn char_cast(obj: Object) -> Object {
        type P = Primitive;
        let c = match obj.primitive {
            P::Null | P::Float(_) | P::Class(_) | P::Function(_) | P::Type(_) | P::List(_) => return Object::make_null(),
            P::Bool(b) => if b { 'T' } else { 'F' },
            P::Int(i) => match char::from_u32(i as u32) {
                Some(c) => c,
                None => return Object::make_null(),
            },
            P::Char(c) => c,
        };
        Object::make_char(c)
    }
    fn type_cast(obj: Object) -> Object {
        Object::make_type(obj.get_type())
    }
    fn list_cast(obj: Object) -> Object {
        Object::make_list(vec![obj.clone()])
    }
    fn func_cast(obj: Object) -> Object {
        type P = Primitive;
        match obj.primitive {
            P::Function(f) => Object::make_func_dir(f),
            _ => return Object::make_null(),
        }
    }
    fn class_cast(obj: Object) -> Object {
        type P = Primitive;
        match obj.primitive {
            P::Class(_) => unimplemented!(),
            _ => return Object::make_null(),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct List {
    pub ltype: Option<PrimType>,
    pub elems: Vec<Object>,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(unpredictable_function_pointer_comparisons)]
pub enum Function {
    RustFn(RustFunc),
    LangFn(Vec<String>, Box<Expression>),
}

#[derive(Clone)]
pub struct Object {
    pub primitive: Primitive,
    pub vtable: &'static VTable,
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
    pub fn make_func(params: Vec<String>, body: Box<Expression>) -> Self {
        Self {
            primitive: Primitive::Function(Function::LangFn(params, body)),
            vtable: &vtable::function_vtable::VTABLE,
        }
    }
    pub fn make_func_dir(func: Function) -> Self {
        Self {
            primitive: Primitive::Function(func),
            vtable: &vtable::function_vtable::VTABLE,
        }
    }
    fn make_rust_func(func: RustFunc) -> Self {
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
        write!(f, "{}", match self.primitive {
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
                ref elems
            }) => elems
                .iter()
                .map(|c| match c.primitive {
                    Primitive::Char(c) => c,
                    _ => unreachable!(),
                }).collect(),
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
        })
    }
}
impl Primitive {
    fn is_truthy(&self) -> bool {
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

#[derive(Clone, Debug, PartialEq)]
pub struct IfExpr {
    pub condition: Box<Expression>,
    pub to_resolve: Box<Expression>,
    pub to_else: Box<Expression>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct LoopExpr {
    pub condition: Box<Expression>,
    pub to_resolve: Box<Expression>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnOpExpr {
    pub op: Operator,
    pub operand: Box<Expression>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinOpExpr {
    pub op: Operator,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

// pub type RustFunc = fn(Vec<Object>) -> Object;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RustFunc {
    num_args: Option<i64>,
    fn_ptr: fn(Vec<Object>) -> Object,
}
impl RustFunc {
    pub const fn new(num_args: Option<i64>, fn_ptr: fn(Vec<Object>) -> Object) -> Self {
        Self { num_args, fn_ptr }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Value(Object),
    Variable(String),
    MakeVar(String, VarData),
    If(IfExpr),
    Loop(LoopExpr),
    UnOp(UnOpExpr),
    BinOp(BinOpExpr),
    BlockExpr(Vec<Expression>),
    ListExpr(Vec<Expression>),
    Call(Vec<Expression>),
    Return(Box<Expression>),
}

impl Default for Expression {
    fn default() -> Self {
        Expression::Value(Object::default())
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct VarData {
    pub is_const: bool,
    pub is_init: bool,
    pub value: Box<Object>,
}

impl VarData {
    fn make_constant(value: Object) -> Self {
        Self {
            is_const: true,
            is_init: true,
            value: Box::new(value),
        }
    }
}
#[derive(Debug)]
pub struct Scope {
    pub vars: HashMap<String, VarData>,
}

impl Default for Scope {
    fn default() -> Self {
        let mut initial_vars = HashMap::new();
        initial_vars.insert(
            String::from("true"),
            VarData::make_constant(Object::make_bool(true)),
        );
        initial_vars.insert(
            String::from("false"),
            VarData::make_constant(Object::make_bool(false)),
        );
        initial_vars.insert(
            String::from("null"),
            VarData::make_constant(Object::make_null()),
        );
        type PT = PrimType;
        let types = [
            ("Null", PT::Null),
            ("Bool", PT::Bool),
            ("Int", PT::Int),
            ("Float", PT::Float),
            ("Char", PT::Char),
            ("List", PT::List),
            ("Function", PT::Function),
            ("Instance", PT::Class),
            ("Type", PT::Type),
        ];
        for (name, typ) in types {
            initial_vars.insert(name.to_string(), VarData::make_constant(Object::make_type(typ)));
        }
        for (name, func) in crate::stdlib::FUNCS {
            initial_vars.insert(
                name.to_string(),
                VarData::make_constant(Object::make_rust_func(*func)),
            );
        }

        Self { vars: initial_vars }
    }
}

impl Expression {
    pub fn resolve(self) -> Object {
        self.evaluate(&mut Scope::default())
    }
    pub fn evaluate(self, scope: &mut Scope) -> Object {
        match self {
            Self::Value(data) => data,
            Self::Variable(name) => {
                let varinfo = scope.vars.entry(name.clone()).or_default();
                if !varinfo.is_init {
                    panic!("Reading uninitialized variable: {name}");
                }
                (*varinfo.value).clone()
            }
            Self::If(if_obj) => {
                let should_execute = if_obj.condition.evaluate(scope);

                // Add bool conversion logic later

                if should_execute.primitive.is_truthy() {
                    if_obj.to_resolve.evaluate(scope)
                } else {
                    if_obj.to_else.evaluate(scope)
                }
            }
            Self::Loop(loop_obj) => {
                let mut result = Object::default();
                loop {
                    let should_loop = loop_obj.condition.clone().evaluate(scope);
                    if should_loop.primitive.is_truthy() {
                        result = loop_obj.to_resolve.clone().evaluate(scope);
                    } else {
                        break result;
                    }
                }
            }
            Self::UnOp(unop) => {
                let operand = unop.operand.evaluate(scope);
                let func = pick_unfunc(unop.op, operand.vtable);
                func(operand.primitive)
            }
            Self::BinOp(binop) => {
                if binop.op == Operator::Assign {
                    assign(*binop.left, *binop.right, scope)
                } else {
                    let left_val = (*binop.left).evaluate(scope);
                    let right_val = (*binop.right).evaluate(scope);
                    let func = pick_binfunc(binop.op, left_val.vtable);

                    func(left_val.primitive, right_val.primitive)
                }
            }
            Self::BlockExpr(block) => {
                let mut result = Object::default();
                for expr in block {
                    result = expr.evaluate(scope);
                }
                result
            }
            Self::ListExpr(list) => {
                let l = list.into_iter().map(|e| e.evaluate(scope));
                Object::make_list(l.collect())
            }
            Self::MakeVar(name, config) => {
                if scope.vars.contains_key(&name) {
                    panic!("Can not create existing variable: {name}");
                }
                scope.vars.insert(name, config);
                Object::default()
            }
            Self::Call(vec) => {
                assert!(!vec.is_empty());
                let func = vec[0].clone().evaluate(scope);
                // let Primitive::Function(func) = func.primitive else {
                    // panic!("Can not call a non-function");
                // };
                let args = &vec[1..];

                type P = Primitive;
                match func.primitive {
                    P::Type(ty) => {
                        if args.len() != 1 {
                            panic!("Can not cast more than one value at a time");
                        }
                        let mut vec = vec;
                        let arg = vec.swap_remove(1);
                        ty.cast(arg.evaluate(scope))
                    }
                    P::Function(Function::LangFn(fn_params, fn_body)) => {
                        // assert!(args.len() == fn_params.len());
                        if args.len() != fn_params.len() {
                            panic!("Function called with incorrect number of arguments");
                        }

                        let mut fn_scope = Scope::default();
                        for (name, val) in fn_params.into_iter().zip(vec.into_iter().skip(1)) {
                            let config = VarData {
                                is_const: false,
                                is_init: true,
                                value: Box::new(val.evaluate(scope)),
                            };
                            fn_scope.vars.insert(name, config);
                        }
                        fn_body.evaluate(&mut fn_scope)
                    }
                    P::Function(Function::RustFn(RustFunc { num_args, fn_ptr })) => {
                        if num_args.is_some() && args.len() != num_args.unwrap() as usize {
                            panic!("Function called with {} but takes {} amount of arguments", args.len(), num_args.unwrap());
                        }
                        let args = vec.into_iter().skip(1).map(|arg| arg.evaluate(scope));
                        fn_ptr(args.collect())
                    }
                    _ => panic!("Can not call non-function: {func:?}"),
                }
            }
            Self::Return(_) => todo!(),
        }
    }

    pub fn push(&mut self, new_expr: Self) {
        match self {
            Self::Value(Object {
                primitive: Primitive::Null,
                vtable: _,
            }) => *self = new_expr,
            Self::BlockExpr(exprs) => exprs.push(new_expr),
            _ => {
                let exprs = vec![self.clone(), new_expr];
                *self = Expression::BlockExpr(exprs);
            }
        };
    }

    pub fn is_null_lit(&self) -> bool {
        matches!(
            self,
            Self::Value(Object {
                primitive: Primitive::Null,
                vtable: _
            })
        )
    }
}

fn assign(left: Expression, right: Expression, scope: &mut Scope) -> Object {
    let value = Box::new(right.evaluate(scope));

    let Expression::Variable(name) = left else {
        panic!("Attempt to assign to non-variable");
    };
    let Some(var) = scope.vars.get_mut(&name) else {
        panic!("Variable not created: {name}");
    };
    if var.is_const && var.is_init {
        panic!("Constant({name}) can not be overriden with ({value:?})");
    }

    var.value = value;
    var.is_init = true;

    Object::default()
}
