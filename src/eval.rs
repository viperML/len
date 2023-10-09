use chumsky::primitive::todo;
use tracing::debug;
use tracing_test::traced_test;

use crate::{Ast, FunctionCall, Int, Literal};
use core::panic;
use std::{collections::HashMap, error::Error, fmt::Display, marker::PhantomData, sync::Arc};

use crate::Expression;

#[derive(Debug, Clone)]
struct Object(Arc<ObjectRaw>);

impl Object {
    fn new_function<F>(func: F) -> Self
    where
        F: Fn(Object) -> Object,
    {
        Object(Arc::new(ObjectRaw::Function(Function {
            value: Box::new(func),
        })))
    }

    fn new_int<I>(int: I) -> Self
    where
        I: Into<Int>,
    {
        Object(Arc::new(ObjectRaw::Int(int.into())))
    }
}

#[derive(Debug)]
enum ObjectRaw {
    Int(Int),
    String(String),
    Function(Function),
}

#[derive(Educe)]
#[educe(Debug)]
struct Function {
    #[educe(Debug(ignore))]
    value: Box<dyn Fn(Object) -> Object>,
}

#[test]
#[traced_test]
fn test_eval<'src>() {
    let ast = Ast::FunctionCall(FunctionCall {
        function: Box::new(Ast::Identifier(crate::Identifier {
            name: String::from("+1"),
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
struct Scope<'scope> {
    parent: Option<&'scope Self>,
    bindings: HashMap<String, Object>,
}

impl<'scope> Scope<'scope> {
    fn symbol_lookup<S: AsRef<str>>(&'scope self, symbol: S) -> Object {
        self.bindings
            .get(&symbol.as_ref().to_string())
            .unwrap()
            .clone()
    }
}

fn eval(node: Ast, scope: &Scope) -> EvalResult<Object>
{
    match node {
        Ast::Literal(lit) => match lit {
            Literal::Integer(x) => Ok(Object::new_int(x)),
            _ => Err(EvalError::Todo),
        },
        Ast::Identifier(ident) => Ok(scope.symbol_lookup(&ident.name)),
        _ => Err(EvalError::Todo),
    }
}
