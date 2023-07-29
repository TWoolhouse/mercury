use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Flow {
    In,
    Out,
}

#[derive(Debug)]
pub struct Statement<T>(pub Flow, pub Action<T>);

pub trait FnT<T>: Fn(T) -> T {}
impl<F, T> FnT<T> for F where F: Fn(T) -> T {}
pub type FnOp<T> = Box<dyn FnT<T>>;

#[derive(Debug)]
pub enum Action<T> {
    Transaction { name: String, operator: Operator<T> },
    Variable(Rc<Action<T>>),
    Sequence(Vec<Statement<T>>),
    Set(Vec<Statement<T>>),
}

#[derive(Debug)]
pub enum Operator<T> {
    Value(T),
    Func(FnOp<T>),
}

pub type Store<T> = HashMap<String, Variable<T>>;

#[derive(Debug, Clone)]
pub struct Variable<T> {
    pub flow: Flow,
    pub print: bool,
    pub name: String,
    pub action: Rc<Action<T>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Record<T> {
    pub name: String,
    pub change: T,
}

impl<T> std::fmt::Debug for dyn FnT<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Fn>")
    }
}
