use std::collections::HashMap;

use tracing::debug;
use tracing_test::traced_test;

#[derive(Debug)]
enum Ty {
    Product(HashMap<String, Option<Self>>),
    Sum(HashMap<String, Option<Self>>),
    Primitive(Primitive),
}

#[derive(Debug)]
enum Primitive {
    String,
    Int,
}

#[test]
#[traced_test]
fn test_ty() {
    let x = Ty::Primitive(Primitive::Int);

    let y = Ty::Product(From::from([
        // ..
        (String::from("foo"), Some(Ty::Primitive(Primitive::String))),
    ]));

    let maybe_s = Ty::Sum(From::from([
        (String::from("Some"), Some(Ty::Primitive(Primitive::Int))),
        (String::from("None"), None),
    ]));

    debug!(?x, ?y);
    todo!();
}
