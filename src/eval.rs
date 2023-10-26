use chumsky::primitive::todo;
use tracing::{debug, info};

use crate::ast::{self, Ast, Expr, Literal};
use crate::Int;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::{collections::HashMap, error::Error, fmt::Display, rc::Rc};

#[derive(Debug, Clone)]
pub struct Object {
    ptr: Rc<ObjectRaw>,
}

impl Deref for Object {
    type Target = ObjectRaw;

    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}

impl Object {
    fn new_function<F>(func: F) -> Self
    where
        F: Fn(Object) -> Object + 'static,
    {
        Object {
            ptr: From::from(ObjectRaw::Function(Function {
                value: Rc::new(func),
            })),
        }
    }

    fn new_int<I>(int: I) -> Self
    where
        I: Into<Int>,
    {
        Object {
            ptr: From::from(ObjectRaw::Int(int.into())),
        }
    }

    fn new_string(input: String) -> Self {
        Object {
            ptr: From::from(ObjectRaw::String(input)),
        }
    }
}

#[derive(Debug)]
pub enum ObjectRaw {
    Int(Int),
    String(String),
    Function(Function),
    Product(HashMap<String, Object>),
}

pub struct Function {
    value: Rc<dyn Fn(Object) -> Object>,
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "function@{:p}", self.value)
        // f.debug_struct("Function").field("value", &self.value).finish()
    }
}

#[derive(Debug, Clone)]
pub enum EvalError {
    TypeError,
    Todo,
}

impl Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Error for EvalError {}

type EvalResult<T> = Result<T, EvalError>;

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

        bindings.insert(String::from("id"), Object::new_function(|x| x));

        bindings.insert(
            String::from("+"),
            Object::new_function(|x| {
                let x = x.clone();
                Object::new_function(move |y| match (&*x, &*y) {
                    (ObjectRaw::Int(a), ObjectRaw::Int(b)) => Object::new_int(a + b),
                    _ => todo!(),
                })
            }),
        );

        bindings.insert(
            String::from("-"),
            Object::new_function(|left| {
                let left = left.clone();
                Object::new_function(move |right| match (&*left, &*right) {
                    (ObjectRaw::Int(a), ObjectRaw::Int(b)) => Object::new_int(a - b),
                    _ => todo!(),
                })
            }),
        );

        bindings.insert(
            String::from("*"),
            Object::new_function(|left| {
                let left = left.clone();
                Object::new_function(move |right| match (&*left, &*right) {
                    (ObjectRaw::Int(a), ObjectRaw::Int(b)) => Object::new_int(a * b),
                    _ => todo!(),
                })
            }),
        );

        bindings.insert(
            String::from("$"),
            Object::new_function(|left| {
                let left = left.clone();
                Object::new_function(move |right| match &*left {
                    ObjectRaw::Function(f) => (f.value)(right),
                    _ => todo!(),
                })
            }),
        );

        bindings.insert(
            String::from("inc"),
            Object::new_function(|x| match &*x {
                ObjectRaw::Int(i) => Object::new_int(i + 1),
                _ => todo!(),
            }),
        );

        bindings.insert(
            String::from("get"),
            Object::new_function(|left| {
                let left = left.clone();
                Object::new_function(move |right| match (&*left, &*right) {
                    (ObjectRaw::Product(p), ObjectRaw::String(s)) => {
                        p.get(s).expect("Failed to get element").clone()
                    }
                    _ => todo!(),
                })
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

pub fn eval_expr(ast: Expr, scope: Scope) -> EvalResult<Object> {
    match ast {
        Expr::Literal(lit) => match lit {
            Literal::Integer(x) => Ok(Object::new_int(x)),
            Literal::String(x) => Ok(Object::new_string(x)),
            _ => Err(EvalError::Todo),
        },
        Expr::Identifier(ident) => Ok(scope.symbol_lookup(ident.name).unwrap()),
        Expr::FunctionCall(call) => {
            let function = eval_expr(*call.function, scope.clone()).unwrap();
            let argument = eval_expr(*call.argument, scope.clone()).unwrap();
            match *function {
                ObjectRaw::Function(ref f) => {
                    let result = (f.value)(argument);
                    Ok(result)
                }
                _ => Err(EvalError::TypeError),
            }
        }
        Expr::Todo => todo!(),
        Expr::Product(_) => todo!(),
        Expr::Lambda(ast::Lambda { from, to }) => Ok({
            let parent = scope.clone();
            Object::new_function(move |argument| {
                let mut bindings = HashMap::new();

                bindings.insert(from.name.clone(), argument);

                let inner_scope = Scope::from_raw(RawScope {
                    parent: Some(parent.clone()),
                    bindings,
                });

                eval_expr(*to.clone(), inner_scope).expect("FIXME")
            })
        }),
        // _ => todo!(),
    }
}

pub fn eval(ast: Ast, scope: Scope) -> Option<Scope> {
    match ast {
        Ast::Expr(expr) => {
            let res = eval_expr(expr, scope.clone());
            info!("{:#?}", res);
            None
        }
        Ast::Binding {
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
        Ast::Todo => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{FunctionCall, Identifier};

    use super::*;
    use tracing::debug;
}
