use crate::{ops::FnOp, Error};

use lazy_static::lazy_static;

#[macro_use]
mod macros;
mod tax;

#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    Number(f64),
    String(String),
}

type Result = std::result::Result<(String, FnOp<f64>), Error>;

pub fn functions() -> &'static [(&'static str, Vec<&'static str>)] {
    &*ARRAY
}

macros::builtins![
    ("tax.ni", tax::ni, "class"),
    ("tax.inc", tax::inc,),
    ("tax.sfe", tax::sfe, "plan")
];

fn arguments(name: &str) -> &'static [&'static str] {
    ARRAY
        .iter()
        .find(|(n, _)| n == &name)
        .map(|(_, args)| args.as_slice())
        .expect("Argument should be in the array from builtins! macro")
}
