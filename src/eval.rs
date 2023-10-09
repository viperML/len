use chumsky::primitive::todo;
use tracing::debug;
use tracing_test::traced_test;

use crate::{Ast, FunctionCall, Int, Literal};
use core::panic;
use std::ops::Deref;
use std::{
    collections::HashMap, error::Error, fmt::Display, marker::PhantomData, rc::Rc, sync::Arc,
};

use crate::Expression;

#[derive(Debug, Clone)]
struct Object<'o>(Rc<ObjectRaw<'o>>);

impl<'o> Deref for Object<'o> {
    type Target = ObjectRaw<'o>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'o> Object<'o> {
    fn new_function<F>(func: F) -> Self
    where
        F: Fn(Object) -> Object + 'o,
    {
        Object(From::from(ObjectRaw::Function(Function {
            value: Box::new(func),
        })))
    }

    fn new_int<I>(int: I) -> Self
    where
        I: Into<Int>,
    {
        Object(From::from(ObjectRaw::Int(int.into())))
    }
}

#[derive(Debug)]
enum ObjectRaw<'o> {
    Int(Int),
    String(String),
    Function(Function<'o>),
}

#[derive(Educe)]
#[educe(Debug)]
struct Function<'f> {
    #[educe(Debug(ignore))]
    value: Box<dyn Fn(Object) -> Object + 'f>,
}

#[test]
#[traced_test]
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
        parent: None,
        bindings,
    };

    let res = eval(ast, &root_scope);
    debug!(?res);

    todo!();
}

#[derive(Debug, Clone)]
enum EvalError {
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
struct Scope<'scope, 'o> {
    parent: Option<&'scope Self>,
    bindings: HashMap<String, Object<'o>>,
}

impl<'scope, 'o> Scope<'scope, 'o> {
    fn symbol_lookup<S: AsRef<str>>(&'scope self, symbol: S) -> Object {
        self.bindings
            .get(&symbol.as_ref().to_string())
            .unwrap()
            .clone()
    }
}

fn eval<'scope, 'o>(node: Ast, scope: &'scope Scope) -> EvalResult<Object<'o>>
where
    'scope: 'o,
{
    match node {
        Ast::Literal(lit) => match lit {
            Literal::Integer(x) => Ok(Object::new_int(x)),
            _ => Err(EvalError::Todo),
        },
        Ast::Identifier(ident) => Ok(scope.symbol_lookup(&ident.name)),
        Ast::FunctionCall(call) => {
            let function = eval(*call.function, &scope).unwrap();
            let argument = eval(*call.argument, &scope).unwrap();
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
