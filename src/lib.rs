mod builtin;
mod ops;
mod pump;
mod syntax;

pub use crate::{builtin::functions, ops::Store};
use ops::Record;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Parsing Error: {0}")]
    Parse(#[from] Box<syntax::error::Parse>),
    #[error("Undefined Variable: {0}")]
    UndefinedVariable(String),
    #[error("Missing Argument: '{0}'")]
    ArgumentMissing(&'static str),
    #[error("Invalid Argument: '{0}'")]
    ArgumentInvalid(String),
    #[error("Invalid Argument Type: '{0}'")]
    ArgumentInvalidType(&'static str),
}

pub fn parse(input: &str) -> Result<ops::Store<f64>, Error> {
    syntax::compile(syntax::parse(input)?)
}
pub fn evaluate(store: &ops::Store<f64>) -> Vec<(&String, (f64, Vec<Record<f64>>))> {
    store
        .iter()
        .filter_map(|(_, var)| var.print.then(|| (&var.name, pump::evaluate(&var.action))))
        .collect()
}
