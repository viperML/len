use crate::ast::Identifier;
use crate::ast::{Ast, FunctionCall, Literal};
use crate::Int;
use std::ops::Deref;
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

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl Error for EvalError {}

type EvalResult<T> = Result<T, EvalError>;

#[derive(Debug)]
pub struct Scope<'parent> {
    parent: Option<&'parent Self>,
    bindings: HashMap<String, Object>,
}

impl<'parent> Scope<'parent> {
    pub fn std() -> Self {
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

impl<'parent> Scope<'parent> {
    fn symbol_lookup<S: AsRef<str>>(&self, symbol: S) -> Object {
        self.bindings
            .get(&symbol.as_ref().to_string())
            .unwrap()
            .clone()
    }
}

pub fn eval(ast: Ast, scope: &Scope) -> EvalResult<Object> {
    match ast {
        Ast::Literal(lit) => match lit {
            Literal::Integer(x) => Ok(Object::new_int(x)),
            Literal::String(x) => Ok(Object::new_string(x)),
            _ => Err(EvalError::Todo),
        },
        Ast::Identifier(ident) => Ok(scope.symbol_lookup(&ident.name)),
        Ast::FunctionCall(call) => {
            let function = eval(*call.function, scope).unwrap();
            let argument = eval(*call.argument, scope).unwrap();
            match *function {
                ObjectRaw::Function(ref f) => {
                    let result = (f.value)(argument);
                    Ok(result)
                }
                _ => Err(EvalError::TypeError),
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use tracing::debug;

    // #[test]
    // #[traced_test]
    fn test_eval<'src>() {
        let ast = Ast::FunctionCall(FunctionCall {
            function: Box::new(Ast::Identifier(Identifier {
                name: String::from("id"),
            })),
            argument: Box::new(Ast::Literal(Literal::Integer(1.into()))),
        });

        let mut bindings = HashMap::new();

        let func_id = Object::new_function(|x| x);
        bindings.insert(String::from("id"), func_id);

        let root_scope = Scope {
            parent: None,
            bindings,
        };

        let res = eval(ast, &root_scope);
        debug!(?res);

        todo!();
    }
}
