use chumsky::primitive::todo;
use thiserror::Error;
use tracing::{debug, info};

use crate::ast::{self};
use crate::Int;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::{collections::HashMap, error::Error, fmt::Display, rc::Rc};

#[derive(Debug, Clone)]
pub struct Object {
    ptr: Rc<RawObject>,
}

impl Deref for Object {
    type Target = RawObject;

    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}

impl Object {
    fn new_function<F>(func: F) -> Self
    where
        F: Fn(Object) -> ExprResult<Object> + 'static,
    {
        Object {
            ptr: From::from(RawObject::Function(Function {
                value: Box::from(func),
            })),
        }
    }

    fn new_int<I>(int: I) -> Self
    where
        I: Into<Int>,
    {
        Object {
            ptr: From::from(RawObject::Int(int.into())),
        }
    }

    fn new_string(input: String) -> Self {
        Object {
            ptr: From::from(RawObject::String(input)),
        }
    }
}

#[derive(Debug)]
pub enum RawObject {
    Int(Int),
    String(String),
    Function(Function),
    Product(HashMap<String, Object>),
}


pub struct Function {
    value: Box<dyn Fn(Object) -> ExprResult<Object>>,
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "function @ {:p}", self.value)
    }
}

#[derive(Debug, Clone, Error)]
pub enum ExprError {
    #[error("Type error")]
    TypeError,
    #[error("Todo")]
    Todo,
}

type ExprResult<T> = Result<T, ExprError>;

#[derive(Debug, Clone)]
pub struct RawScope {
    pub parent: Option<Scope>,
    pub bindings: HashMap<String, Object>,
}

#[derive(Debug, Clone)]
pub struct Scope(Rc<RawScope>);

impl RawScope {
    fn std() -> Self {
        let mut bindings = HashMap::new();

        bindings.insert(
            String::from("+"),
            Object::new_function(|x| {
                let x = x.clone();
                Ok(Object::new_function(move |y| match (&*x, &*y) {
                    (RawObject::Int(a), RawObject::Int(b)) => Ok(Object::new_int(a + b)),
                    _ => todo!(),
                }))
            }),
        );

        bindings.insert(
            String::from("-"),
            Object::new_function(|left| {
                let left = left.clone();
                Ok(Object::new_function(move |right| match (&*left, &*right) {
                    (RawObject::Int(a), RawObject::Int(b)) => Ok(Object::new_int(a - b)),
                    _ => todo!(),
                }))
            }),
        );

        bindings.insert(
            String::from("*"),
            Object::new_function(|left| {
                let left = left.clone();
                Ok(Object::new_function(move |right| match (&*left, &*right) {
                    (RawObject::Int(a), RawObject::Int(b)) => Ok(Object::new_int(a * b)),
                    _ => todo!(),
                }))
            }),
        );

        bindings.insert(
            String::from("$"),
            Object::new_function(|left| {
                let left = left.clone();
                Ok(Object::new_function(move |right| match &*left {
                    RawObject::Function(f) => (f.value)(right),
                    _ => todo!(),
                }))
            }),
        );

        bindings.insert(
            String::from("inc"),
            Object::new_function(|x| {
                todo!()
            }),
        );

        bindings.insert(
            String::from("get"),
            Object::new_function(|left| {
                let left = left.clone();
                Ok(Object::new_function(move |right| match (&*left, &*right) {
                    (RawObject::Product(p), RawObject::String(s)) => {
                        Ok(p.get(s).expect("Failed to get element").clone())
                    }
                    _ => todo!(),
                }))
            }),
        );

        Self {
            parent: None,
            bindings,
        }
    }
}

impl Scope {
    pub fn std() -> Self {
        Scope(From::from(RawScope::std()))
    }

    #[tracing::instrument(ret, level = "debug")]
    pub fn symbol_lookup<S: AsRef<str> + fmt::Debug>(&self, symbol: S) -> Option<Object> {
        match self.bindings.get(&symbol.as_ref().to_string()) {
            Some(o) => Some(o.clone()),
            None => match self.parent.clone() {
                None => None,
                Some(p) => p.symbol_lookup(symbol),
            },
        }
    }

    pub fn from_raw(raw: RawScope) -> Self {
        Self(From::from(raw))
    }
}

impl Deref for Scope {
    type Target = RawScope;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn eval_expr(ast: ast::Expr, scope: Scope) -> ExprResult<Object> {
    match ast {
        ast::Expr::Literal(lit) => match lit {
            ast::Literal::Integer(x) => Ok(Object::new_int(x)),
            ast::Literal::String(x) => Ok(Object::new_string(x)),
            _ => Err(ExprError::Todo),
        },
        ast::Expr::Identifier(ident) => Ok(scope.symbol_lookup(ident.name).unwrap()),
        ast::Expr::FunctionCall(call) => {
            let function = eval_expr(*call.function, scope.clone()).unwrap();
            let argument = eval_expr(*call.argument, scope.clone()).unwrap();
            match *function {
                RawObject::Function(ref f) => (f.value)(argument),
                _ => Err(ExprError::TypeError),
            }
        }
        ast::Expr::Todo => todo!(),
        ast::Expr::Product(_) => todo!(),
        ast::Expr::Lambda(ast::Lambda { from, to }) => {
            let parent = scope.clone();
            Ok(Object::new_function(move |argument| {
                let mut bindings = HashMap::new();

                bindings.insert(from.name.clone(), argument);

                let inner_scope = Scope::from_raw(RawScope {
                    parent: Some(parent.clone()),
                    bindings,
                });

                eval_expr(*to.clone(), inner_scope)
            }))
        }
        // _ => todo!(),
    }
}

pub fn eval(ast: ast::Ast, scope: Scope) -> Option<Scope> {
    match ast {
        ast::Ast::Expr(expr) => {
            let res = eval_expr(expr, scope.clone());
            info!("{:#?}", res);
            None
        }
        ast::Ast::Binding {
            lhs: ident,
            rhs: expr,
        } => {
            let res = eval_expr(expr, scope.clone());
            info!("{:#?}", res);
            let res = res.unwrap();

            let mut new_bindings = scope.0.bindings.clone();
            new_bindings.insert(ident.name, res);

            Some(Scope(Rc::from(RawScope {
                parent: scope.parent.clone(),
                bindings: new_bindings,
            })))
        }
        ast::Ast::Todo => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{FunctionCall, Identifier};

    use super::*;
    use tracing::debug;
}
