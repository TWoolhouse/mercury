use std::fmt::Debug;

use crate::account;

pub type Operation = Box<dyn Fn(&mut account::CtxMut) -> f64>;

pub struct Statement {
    pub from: account::ID,
    pub to: account::ID,
    pub func: Operation,
    pub label: Option<String>,
}

impl Debug for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Statement")
            .field("from", &self.from)
            .field("to", &self.to)
            .field("label", &self.label)
            .finish()
    }
}

#[derive(Debug)]
pub enum Statements {
    List(account::ID, Vec<Statements>),
    Set(account::ID, Vec<Statements>),
    Single(Statement),
}
