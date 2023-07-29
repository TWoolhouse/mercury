mod parsing;

pub use self::parsing::Rule;
use self::parsing::*;

use crate::{
    builtin::{lookup, Arg},
    ops::{Action, Flow, Operator, Statement, Store, Variable},
};
use std::rc::Rc;

use super::Error;
type Result<T> = std::result::Result<T, Error>;

pub mod error {
    pub type Parse = pest::error::Error<super::Rule>;
}

pub fn parse(input: &str) -> Result<List> {
    use pest::Parser;
    Ok(parsing::Mercury::parse(Rule::root, input)
        .map_err(Box::new)?
        .next()
        .unwrap()
        .into_child()
        .into_inner())
}
pub fn compile(tokens: List) -> Result<Store<f64>> {
    let mut store = Store::default();
    tokens
        .map(|node| stmt(node, &mut store))
        .collect::<Result<Vec<_>>>()?;
    Ok(store)
}

pub(super) fn stmt(node: Node, store: &mut Store<f64>) -> Result<Statement<f64>> {
    let node = node.into_child();
    match node.as_rule() {
        Rule::assignment => {
            let print = node.as_str().chars().next().unwrap();
            let mut nodes = node.into_inner();
            let name = nodes.next().unwrap();
            let value = nodes.next().unwrap();

            let stmt = expr(value, store)?;
            let var = Rc::new(stmt.1);
            let display = name.as_str().to_string();
            store.insert(
                identifier(name),
                Variable {
                    name: display,
                    flow: stmt.0,
                    print: print == '£',
                    action: var.clone(),
                },
            );

            Ok(Statement(stmt.0, Action::Variable(var)))
        }
        Rule::expr => expr(node, store),
        _ => unreachable!("Unexpected rule: {:?}", node.as_rule()),
    }
}

pub(super) fn expr(node: Node, store: &mut Store<f64>) -> Result<Statement<f64>> {
    let flow = match node.as_str().chars().next().unwrap() {
        '+' => Some(Flow::In),
        '-' => Some(Flow::Out),
        _ => None,
    };
    let node = node.into_child();
    Ok(match node.as_rule() {
        Rule::list => {
            let collection = node.as_str().chars().next().unwrap();
            let children = node
                .into_child()
                .into_inner()
                .map(|node| stmt(node, store))
                .collect::<Result<Vec<_>>>()?;

            let flow = flow.unwrap_or(Flow::In);
            match collection {
                '[' => Statement(flow, Action::Sequence(children)),
                '{' => Statement(flow, Action::Set(children)),
                _ => unreachable!(),
            }
        }
        Rule::node => {
            let mut nodes = node.into_inner();
            let name = nodes.next().unwrap();
            let node = nodes.next().unwrap();
            Statement(
                flow.unwrap_or(Flow::Out),
                match node.as_rule() {
                    Rule::node_value => Action::Transaction {
                        name: node_name(name),
                        operator: node_value(node),
                    },
                    Rule::node_func => todo!(),
                    _ => unreachable!("Unexpected rule: {:?}", node.as_rule()),
                },
            )
        }
        Rule::identifier => {
            let name = identifier(node);
            let var = store.get(&name).ok_or(Error::UndefinedVariable(name))?;
            Statement(
                flow.unwrap_or(var.flow),
                Action::Variable(var.action.clone()),
            )
        }
        Rule::builtin => {
            let mut nodes = node.into_inner();
            let name = nodes.next().unwrap();

            let ident = identifier(name);
            let args: Vec<_> = nodes
                .map(|node| {
                    let child = node.clone().into_child();
                    match child.as_rule() {
                        Rule::value => Arg::Number(child.as_str().parse().unwrap()),
                        _ => Arg::String(node_name(node)),
                    }
                })
                .collect();

            let (name, func) = lookup(&ident, &args)?;

            Statement(
                flow.unwrap_or(Flow::Out),
                Action::Transaction {
                    name,
                    operator: Operator::Func(func),
                },
            )
        }
        _ => unreachable!("Unexpected rule: {:?}", node.as_rule()),
    })
}

fn identifier(node: Node) -> String {
    node.as_str()
        .replace(|c: char| c.is_whitespace() || c == '_', "")
        .to_lowercase()
}

fn node_name(node: Node) -> String {
    let node = node.into_child();
    match node.as_rule() {
        Rule::name => node.as_str().trim().into(),
        Rule::string => node.into_child().as_str().trim().into(),
        _ => unreachable!("Unexpected rule: {:?}", node.as_rule()),
    }
}

fn node_value(node: Node) -> Operator<f64> {
    let mut nodes = node.into_inner();
    let value = nodes.next().unwrap();

    let value: f64 = value.as_str().parse().unwrap();

    let variant = nodes.next().unwrap().into_child();
    match variant.as_rule() {
        Rule::type_rate => {
            let rate: f64 = match variant.as_str() {
                "d" => 365.0,
                "w" => 52.0,
                "m" => 12.0,
                "q" => 4.0,
                "y" => 1.0,
                "wd" => 260.0,
                "we" => 105.0,
                _ => unreachable!("Unexpected rate: {:?}", variant.as_str()),
            };
            Operator::Value(value * rate)
        }
        Rule::type_percentage => Operator::Func(Box::new(move |x| x * value / 100.0)),
        _ => unreachable!("Unexpected rule: {:?}", variant.as_rule()),
    }
}
