use tracing::debug;

use crate::{Ast, FunctionCall, Int, Literal};

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
        F: Fn(Object) -> Object,
    {
        Object {
            ptr: From::from(ObjectRaw::Function(Function {
                value: Box::new(func),
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

#[derive(Educe)]
#[educe(Debug)]
pub struct Function {
    #[educe(Debug(ignore))]
    value: Box<dyn Fn(Object) -> Object>,
}

// #[test]
// #[traced_test]
fn test_eval<'src>() {
    let ast = Ast::FunctionCall(FunctionCall {
        function: Box::new(Ast::Identifier(crate::Identifier {
            name: String::from("id"),
        })),
        argument: Box::new(Ast::Literal(Literal::Integer(1.into()))),
    });

    let mut bindings = HashMap::new();

    let func_id = Object::new_function(|x| x);
    bindings.insert(String::from("id"), func_id);

    let root_scope = Scope {
        // parent: None,
        bindings,
    };

    let res = eval(ast, &root_scope);
    debug!(?res);

    todo!();
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
pub struct Scope {
    // parent: Option<&'scope Self>,
    bindings: HashMap<String, Object>,
}

impl Scope {
    pub fn std() -> Self {
        let mut bindings = HashMap::new();
        let func_id = Object::new_function(|x| x);


        bindings.insert(
            String::from("+"),
            Object::new_function(|x| {
                let x = x.clone();
                Object::new_function(move |y| match (&*x, &*y) {

                    _ => todo!(),
                })
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
            // parent: None,
            bindings,
        }
    }
}

impl Scope
{
    fn symbol_lookup<S: AsRef<str>>(&self, symbol: S) -> Object {
        self.bindings
            .get(&symbol.as_ref().to_string())
            .unwrap()
            .clone()
    }
}

pub fn eval(ast: Ast, scope: &Scope) -> EvalResult<Object>
{
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
