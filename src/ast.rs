use crate::{   
    util::{Operator, BadFnArgCnt, MyriaErr, MyriaRes},
    obj::{Object, PrimType, Primitive, RustFunc},
    vtable::{pick_binfunc, pick_unfunc},
};
use std::collections::HashMap;

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
pub struct TryExpr {
    pub body: Box<Expression>,
    pub catch: Box<Expression>,
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

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Value(Object),
    Variable(String),
    MakeVar(String, VarData),
    If(IfExpr),
    Loop(LoopExpr),
    Try(TryExpr),
    Throw(Box<Expression>),
    UnOp(UnOpExpr),
    BinOp(BinOpExpr),
    BlockExpr(Vec<Expression>),
    ListExpr(Vec<Expression>),
    Call(Vec<Expression>),
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
    pub value: Object,
}

impl VarData {
    pub fn make_constant(value: Object) -> Self {
        Self {
            is_const: true,
            is_init: true,
            value,
        }
    }
    pub fn make_var(value: Object) -> Self {
        Self {
            is_const: false,
            is_init: true,
            value,
        }
    }
    fn can_modify(&self) -> bool {
        !self.is_const || !self.is_init
    }
    fn set(&mut self, value: Object) -> Result<(), MyriaErr> {
        if !self.can_modify() {
            Err(MyriaErr::VariableNotMut("{}".into()))
        } else {
            self.value = value;
            self.is_init = true;
            Ok(())
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Scope {
    pub vars: HashMap<String, VarData>,
}

impl Default for Scope {
    fn default() -> Self {
        let mut ret = Self::empty();
        ret.push_const("true".into(), Object::make_bool(true));
        ret.push_const("false".into(), Object::make_bool(false));
        ret.push_const("null".into(), Object::make_null());

        type PT = PrimType;
        let types = [
            ("Null", PT::Null),
            ("Bool", PT::Bool),
            ("Int", PT::Int),
            ("Float", PT::Float),
            ("Char", PT::Char),
            ("List", PT::List),
            ("Function", PT::Function),
            ("Instance", PT::Instance),
            ("Type", PT::Type),
        ];
        for (name, typ) in types {
            ret.push_const(name.into(), Object::make_type(typ));
        }
        for (name, func) in crate::stdlib::FUNCS {
            ret.push_const((*name).into(), Object::make_rust_func(*func));
        }

        ret
    }
}
impl Scope {
    pub fn empty() -> Self {
        Self {
            vars: Default::default(),
        }
    }
    pub fn push_var(&mut self, name: String, value: Object) {
        self.vars.insert(name, VarData::make_var(value));
    }
    pub fn push_const(&mut self, name: String, value: Object) {
        self.vars.insert(name, VarData::make_constant(value));
    }
    pub fn get(&self, name: &str) -> Result<VarData, MyriaErr> {
        // dbg!(name);
        if !name.contains('.') {
            self.get_once(name)
        } else {
            let Some((base, rest)) = name.split_once('.') else {
                unreachable!();
            };
            let base_var = self.get_once(base)?;
            let Primitive::Instance(fields) = base_var.value.primitive else {
                return Err(MyriaErr::InvalidOperation(
                    "Can not get fields of non-Instance".into(),
                ));
            };
            fields.get(rest)
        }
    }
    fn get_once(&self, name: &str) -> Result<VarData, MyriaErr> {
        self.vars
            .get(name)
            .cloned()
            .ok_or(MyriaErr::VariableDNE(name.to_string()))
    }
    fn get_once_mut(&mut self, name: &str) -> Result<&mut VarData, MyriaErr> {
        self.vars
            .get_mut(name)
            .ok_or(MyriaErr::VariableDNE(name.to_string()))
    }
    fn var_exists(&self, name: &str) -> bool {
        if !name.contains('.') {
            self.vars.contains_key(name)
        } else {
            let Some((base, rest)) = name.split_once('.') else {
                unreachable!();
            };
            let Ok(base_var) = self.get_once(base) else {
                return false;
            };
            let Primitive::Instance(fields) = base_var.value.primitive else {
                // panic!("Can not get fields of non-Instance");
                return false;
            };
            fields.var_exists(rest)
        }
    }
    fn set(&mut self, name: &str, value: Object) -> Result<(), MyriaErr> {
        if !name.contains('.') {
            self.set_once(name, value)
        } else {
            let Some((base, rest)) = name.split_once('.') else {
                unreachable!();
            };
            let base_var = self.get_once_mut(base)?;
            let Primitive::Instance(ref mut fields) = base_var.value.primitive else {
                return Err(MyriaErr::InvalidOperation(
                    "Can not set fields of non-Instance".into(),
                ));
            };
            fields.set(rest, value)
        }
    }
    fn set_once(&mut self, name: &str, value: Object) -> Result<(), MyriaErr> {
        let config = self
            .vars
            .get_mut(name)
            .ok_or(MyriaErr::VariableDNE(name.to_string()))?;
        if !config.can_modify() {
            return Err(MyriaErr::VariableNotMut(name.into()));
        }
        config.set(value)
    }
    fn create_with_config(&mut self, name: &str, config: VarData) -> Result<(), MyriaErr> {
        if !name.contains('.') {
            self.vars.insert(name.into(), config);
            Ok(())
        } else {
            let Some((base, rest)) = name.split_once('.') else {
                unreachable!();
            };
            let base_var = self.get_once_mut(base)?;
            let Primitive::Instance(ref mut fields) = base_var.value.primitive else {
                return Err(MyriaErr::InvalidOperation(
                    "Can not create field on non-Instance".into(),
                ));
            };
            // dbg!(rest, &config);
            fields.create_with_config(rest, config)
            // dbg!(&fields);
        }
    }
}

impl Expression {
    // pub fn resolve(self) -> Object {
    //     self.evaluate(&mut Scope::default())
    // }
    pub fn evaluate(self, scope: &mut Scope) -> MyriaRes {
        match self {
            Self::Value(data) => Ok(data),
            Self::Variable(name) => {
                let varinfo = scope.get(&name)?;
                if !varinfo.is_init {
                    return Err(MyriaErr::VariableNotInit(name));
                }
                Ok(varinfo.value)
            }
            Self::If(if_obj) => {
                let should_execute = if_obj.condition.evaluate(scope)?;

                if should_execute.primitive.is_truthy() {
                    if_obj.to_resolve.evaluate(scope)
                } else {
                    if_obj.to_else.evaluate(scope)
                }
            }
            Self::Loop(loop_obj) => {
                let mut result = Object::default();
                loop {
                    let should_loop = loop_obj.condition.clone().evaluate(scope)?;
                    if should_loop.primitive.is_truthy() {
                        result = loop_obj.to_resolve.clone().evaluate(scope)?;
                    } else {
                        break Ok(result);
                    }
                }
            }
            Self::Try(try_obj) => try_obj
                .body
                .evaluate(scope)
                .or_else(|_| try_obj.catch.evaluate(scope)),
            Self::Throw(throw_obj) => {
                let value = throw_obj.evaluate(scope)?;
                Err(MyriaErr::Thrown(value))
            }
            Self::UnOp(unop) => {
                let operand = unop.operand.evaluate(scope)?;
                let func = pick_unfunc(unop.op, operand.vtable);
                func(operand.primitive)
            }
            Self::BinOp(binop) => {
                if binop.op == Operator::Assign {
                    assign(*binop.left, *binop.right, scope)
                } else {
                    let left_val = binop.left.evaluate(scope)?;
                    let right_val = binop.right.evaluate(scope)?;
                    let func = pick_binfunc(binop.op, left_val.vtable);

                    func(left_val.primitive, right_val.primitive)
                }
            }
            Self::BlockExpr(block) => {
                let mut result = Object::default();
                for expr in block {
                    result = expr.evaluate(scope)?;
                }
                Ok(result)
            }
            Self::ListExpr(list) => {
                let l = list
                    .into_iter()
                    .map(|e| e.evaluate(scope))
                    .collect::<Result<Vec<Object>, MyriaErr>>()?;
                Ok(Object::make_list(l))
            }
            Self::MakeVar(name, config) => {
                if scope.var_exists(&name) {
                    Err(MyriaErr::VariableAlreadyExists(name))
                } else {
                    scope
                        .create_with_config(&name, config)
                        .map(|_| Object::default())
                }
            }

            Self::Call(vec) => function_call(vec, scope),
        }
    }

    pub fn push(&mut self, new_expr: Self) {
        match self {
            Self::Value(Object {
                primitive: Primitive::Null,
                ..
            }) => *self = new_expr,
            Self::BlockExpr(exprs) => match new_expr {
                Self::BlockExpr(other_block) => exprs.extend(other_block),
                other_expr => exprs.push(other_expr),
            },
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

fn assign(left: Expression, right: Expression, scope: &mut Scope) -> MyriaRes {
    let value = right.evaluate(scope)?;
    let Expression::Variable(name) = left else {
        return Err(MyriaErr::InvalidOperation(
            "Can not assign to non-variable".into(),
        ));
    };
    scope.set(&name, value)?;

    Ok(Object::default())
}

fn function_call(vec: Vec<Expression>, scope: &mut Scope) -> MyriaRes {
    use crate::obj::{Function, LangFunc};

    assert!(!vec.is_empty());
    let func = vec[0].clone().evaluate(scope)?;
    let args = &vec[1..];

    type P = Primitive;
    match func.primitive.clone() {
        P::Type(ty) => {
            if args.len() != 1 {
                return Err(MyriaErr::InvalidOperation(
                    "Can not cast more than one value at a time".into(),
                ));
            }
            let mut vec = vec;
            let arg = vec.swap_remove(1);
            Ok(ty.cast(arg.evaluate(scope)?))
        }
        P::Function(Function::LangFn(lfunc)) => {
            let LangFunc {
                params,
                is_variadic,
                body,
            } = lfunc;
            assert!(!is_variadic);
            if args.len() != params.len() {
                return Err(MyriaErr::BadFunctionArgumentCount(BadFnArgCnt {
                    param_count: params.len(),
                    arg_count: args.len(),
                }));
                // panic!(
                // "Function called with incorrect number of arguments({} instead of {}): {:?}",
                // args.len(),
                // params.len(),
                // args,
                // );
            }

            let mut fn_scope = Scope::default();
            fn_scope.push_const("recurs".into(), func);
            for (name, val) in params.into_iter().zip(vec.into_iter().skip(1)) {
                fn_scope.push_var(name, val.evaluate(scope)?);
            }
            body.evaluate(&mut fn_scope)
        }
        P::Function(Function::RustFn(rf)) => {
            let args = vec
                .into_iter()
                .skip(1)
                .map(|arg| arg.evaluate(scope))
                .collect::<Result<Vec<Object>, MyriaErr>>()?;
            function_call_rust(rf, args)
        }
        _ => Err(MyriaErr::InvalidOperation(
            "Can not call non-function: {func:?}".into(),
        )),
    }
}

pub fn function_call_rust(RustFunc { num_args, fn_ptr }: RustFunc, args: Vec<Object>) -> MyriaRes {
    match num_args {
        Some(arg_count) if arg_count as usize != args.len() => panic!(
            "Function called with {} but takes {} amount of arguments",
            args.len(),
            arg_count,
        ),
        _ => (),
    };
    fn_ptr(args)
}
