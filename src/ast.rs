use crate::gen::Operator;
use crate::obj::{Object, PrimType, Primitive};
use crate::vtable::{pick_binfunc, pick_unfunc};
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
    MakeVar(String, VarData, Option<Object>),
    If(IfExpr),
    Loop(LoopExpr),
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
            value: value,
        }
    }
    pub fn make_var(value: Object) -> Self {
        Self {
            is_const: false,
            is_init: true,
            value: value,
        }
    }
    fn can_modify(&self) -> bool {
        !self.is_const || !self.is_init
    }
    fn set(&mut self, value: Object) -> Result<(), ()> {
        if !self.can_modify() {
            Err(())
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
        let mut ret = Self {
            vars: HashMap::new(),
        };
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
    fn push_var(&mut self, name: String, value: Object) {
        self.vars.insert(name, VarData::make_var(value));
    }
    fn push_const(&mut self, name: String, value: Object) {
        self.vars.insert(name, VarData::make_constant(value));
    }
    fn get(&self, name: &str) -> VarData {
        // dbg!(name);
        if !name.contains('.') {
            self.get_once(name)
        } else {
            let Some((base, rest)) = name.split_once('.') else {
                unreachable!();
            };
            let base_var = self.get_once(base);
            let Primitive::Instance(fields) = base_var.value.primitive else {
                panic!("Can not get fields of non-Instance");
            };
            fields.get(rest)
        }
    }
    fn get_once(&self, name: &str) -> VarData {
        match self.vars.get(name) {
            Some(var) => var.clone(),
            None => panic!("Undeclared variable: {name}"),
        }
    }
    fn get_once_mut(&mut self, name: &str) -> &mut VarData {
        self.vars.get_mut(name).expect("Undeclared variable")
    }
    fn var_exists(&self, name: &str) -> bool {
        if !name.contains('.') {
            self.vars.get(name).is_some()
        } else {
            let Some((base, rest)) = name.split_once('.') else {
                unreachable!();
            };
            let base_var = self.get_once(base);
            let Primitive::Instance(fields) = base_var.value.primitive else {
                panic!("Can not get fields of non-Instance");
            };
            fields.var_exists(rest)
        }
    }
    fn set(&mut self, name: &str, value: Object) {
        if !name.contains('.') {
            self.set_once(name, value);
        } else {
            let Some((base, rest)) = name.split_once('.') else {
                unreachable!();
            };
            let base_var = self.get_once_mut(base);
            let Primitive::Instance(ref mut fields) = base_var.value.primitive else {
                panic!("Can not set fields of non-Instance");
            };
            fields.set(rest, value);
        }
    }
    fn set_once(&mut self, name: &str, value: Object) {
        let config = self
            .vars
            .get_mut(name)
            .expect("Variable {name} does not exist");
        if !config.can_modify() {
            panic!("Can not modify {name}: {config:?}");
        }
        config.set(value).unwrap();
    }
    fn create_with_config(&mut self, name: &str, config: VarData) {
        if !name.contains('.') {
            self.vars.insert(name.into(), config);
        } else {
            let Some((base, rest)) = name.split_once('.') else {
                unreachable!();
            };
            let base_var = self.get_once_mut(base);
            let Primitive::Instance(ref mut fields) = base_var.value.primitive else {
                panic!("Can not create field on non-Instance");
            };
            // dbg!(rest, &config);
            fields.create_with_config(rest, config);
            // dbg!(&fields);
        }
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
                let varinfo = scope.get(&name);
                if !varinfo.is_init {
                    panic!("Reading uninitialized variable: {name}");
                }
                varinfo.value
            }
            Self::If(if_obj) => {
                let should_execute = if_obj.condition.evaluate(scope);

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
                } else if binop.op == Operator::Dot {
                    let left_val = binop.left.evaluate(scope);
                    field_access(left_val.primitive, *binop.right, scope)
                } else {
                    let left_val = binop.left.evaluate(scope);
                    let right_val = binop.right.evaluate(scope);
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
            Self::MakeVar(name, config, None) => {
                if scope.var_exists(&name) {
                    panic!("Can not create existing variable: {name}");
                }
                // scope.create_var(name, value);
                scope.create_with_config(&name, config);
                // scope.vars.insert(name, config);
                Object::default()
            }
            Self::MakeVar(_name, _config, Some(_inst)) => {
                println!("HEREEREREr");
                panic!();
                // let Primitive::Instance(mut inst_scope) = inst.primitive else {
                //     panic!("Fields can only be given to Instances");
                // };
                // if inst_scope.var_exists(&name) {
                //     panic!("Can not create existing variable: {name}");
                // }
                // inst_scope.vars.insert(name, config);
                // Object::default()
            }
            Self::Call(vec) => function_call(vec, scope),
        }
    }

    pub fn push(&mut self, new_expr: Self) {
        match self {
            Self::Value(Object { primitive: Primitive::Null, .. }) => *self = new_expr,
            Self::BlockExpr(exprs) => {
                match new_expr {
                    Self::BlockExpr(other_block) => exprs.extend(other_block.into_iter()),
                    other_expr => exprs.push(other_expr),
                }
            }
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
    let value = right.evaluate(scope);
    let Expression::Variable(name) = left else {
        panic!("Attempt to assign to non-variable");
    };
    if !scope.var_exists(&name) {
        panic!("Variable not created: {name}");
    }
    // dbg!(&name, &value);
    scope.set(&name, value);
    // let var = scope.get_mut(&name);
    // if !var.can_modify() {
    //     panic!("Constant({name}) can not be overriden with ({value:?})");
    // }
    // var.set(value).unwrap();

    Object::default()
}
fn field_access(prim: Primitive, expr: Expression, scope: &mut Scope) -> Object {
    use crate::vtable::list_vtable;
    type P = Primitive;
    match prim.clone() {
        P::List(lst) => list_vtable::index(lst, expr.evaluate(scope)),
        P::Instance(_fields) => {
            todo!();
            // let Expression::Variable(name) = expr else {
            // panic!();
            // };
            // let Some(value) = fields.vars.get(&name) else {
            // panic!("Field {name} does not exist on {prim:?}");
            // };
            // (*value.value).clone()
            // Object::default()
        }
        _ => panic!("{prim:?} has no fields!"),
    }
}
fn function_call(vec: Vec<Expression>, scope: &mut Scope) -> Object {
    use crate::obj::{Function, LangFunc, RustFunc};

    assert!(!vec.is_empty());
    let func = vec[0].clone().evaluate(scope);
    let args = &vec[1..];

    type P = Primitive;
    match func.primitive.clone() {
        P::Type(ty) => {
            if args.len() != 1 {
                panic!("Can not cast more than one value at a time");
            }
            let mut vec = vec;
            let arg = vec.swap_remove(1);
            ty.cast(arg.evaluate(scope))
        }
        P::Function(Function::LangFn(lfunc)) => {
            let LangFunc {
                params,
                is_variadic,
                body,
            } = lfunc;
            assert!(!is_variadic);
            if args.len() != params.len() {
                panic!(
                    "Function called with incorrect number of arguments({} instead of {}): {:?}",
                    args.len(),
                    params.len(),
                    args,
                );
            }

            let mut fn_scope = Scope::default();
            fn_scope.push_const("recurs".into(), func);
            for (name, val) in params.into_iter().zip(vec.into_iter().skip(1)) {
                fn_scope.push_var(name, val.evaluate(scope));
            }
            body.evaluate(&mut fn_scope)
        }
        P::Function(Function::RustFn(RustFunc { num_args, fn_ptr })) => {
            match num_args {
                Some(arg_count) if arg_count as usize != args.len() => panic!(
                    "Function called with {} but takes {} amount of arguments",
                    args.len(),
                    arg_count,
                ),
                _ => (),
            };
            let args = vec.into_iter().skip(1).map(|arg| arg.evaluate(scope));
            fn_ptr(args.collect())
        }
        _ => panic!("Can not call non-function: {func:?}"),
    }
}
