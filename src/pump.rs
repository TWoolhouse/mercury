use crate::ops::{Action, Flow, Operator, Record, Statement};

pub fn evaluate(action: &Action<f64>) -> (f64, Vec<Record<f64>>) {
    let mut transactions = vec![];
    let account = recurse(0.0, &mut transactions, action);
    (account, transactions)
}

fn recurse(account: f64, transactions: &mut Vec<Record<f64>>, action: &Action<f64>) -> f64 {
    match action {
        Action::Transaction { name, operator } => {
            let change = match operator {
                Operator::Value(value) => *value,
                Operator::Func(func) => func(account),
            };
            transactions.push(Record {
                name: name.clone(),
                change,
            });
            change
        }
        Action::Sequence(children) => children
            .into_iter()
            .fold(0.0, |acc, stmt| pump(acc, transactions, stmt)),
        Action::Set(children) => children
            .into_iter()
            .map(|stmt| flow(stmt.0) * recurse(account, transactions, &stmt.1))
            .sum(),
        Action::Variable(var) => recurse(account, transactions, var),
    }
}

fn pump(account: f64, transactions: &mut Vec<Record<f64>>, stmt: &Statement<f64>) -> f64 {
    let change = recurse(account, transactions, &stmt.1);
    match stmt.0 {
        Flow::In => account + change,
        Flow::Out => account - change,
    }
}

fn flow(flow: Flow) -> f64 {
    match flow {
        Flow::In => 1.0,
        Flow::Out => -1.0,
    }
}
