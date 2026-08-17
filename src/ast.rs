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
    Int,
    Float,
    Char,
    Bool,

    Type,
    List,
    Function,
    Class,
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
            Self::Char(_) => true,
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

pub type RustFunc = fn(Vec<Object>) -> Object;

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
#[derive(Debug)]
pub struct Scope {
    pub vars: HashMap<String, VarData>,
}

impl Default for Scope {
    fn default() -> Self {
        let mut initial_vars = HashMap::new();
        initial_vars.insert(
            String::from("true"),
            VarData {
                is_const: true,
                is_init: true,
                value: Box::new(Object::make_bool(true)),
            },
        );
        initial_vars.insert(
            String::from("false"),
            VarData {
                is_const: true,
                is_init: true,
                value: Box::new(Object::make_bool(false)),
            },
        );
        initial_vars.insert(
            String::from("null"),
            VarData {
                is_const: true,
                is_init: true,
                value: Box::new(Object::make_null()),
            },
        );
        for (name, func) in crate::stdlib::FUNCS {
            initial_vars.insert(
                name.to_string(),
                VarData {
                    is_const: true,
                    is_init: true,
                    value: Box::new(Object::make_rust_func(*func)),
                },
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
                let Primitive::Function(func) = func.primitive else {
                    panic!("Can not call a non-function");
                };
                let args = &vec[1..];

                match func {
                    Function::LangFn(fn_params, fn_body) => {
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
                    Function::RustFn(fn_ptr) => {
                        let args = vec.into_iter().skip(1).map(|arg| arg.evaluate(scope));
                        fn_ptr(args.collect())
                    }
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
