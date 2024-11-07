use std::str::FromStr;

use crate::{account, Datestamp, Event, Operation, Statement, Statements};

use super::Schedule;
use parser::Rule;

use self::parser::{List, Node, NodeParent};

mod parser;

#[derive(Debug, PartialEq)]
pub struct Context<'a> {
    pub accounts: &'a mut account::Interner,
    pub date_start: Datestamp,
    pub date_end: Datestamp,
}

fn parse_root(ctx: &mut Context, mut root: List) -> Vec<Event> {
    let mut current: Vec<account::ID> = Default::default();
    root.next()
        .expect("Root must have atleast 1 child")
        .into_inner()
        .flat_map(|node| parse_declaration(ctx, &mut current, node))
        .collect()
}

fn parse_acc_node(accounts: &mut account::Interner, node: Node) -> account::ID {
    accounts.get_or_intern(node.as_str().trim())
}

fn parse_declaration<'a: 'b, 'b>(
    ctx: &mut Context<'a>,
    current: &mut Vec<account::ID>,
    declaration: Node<'b>,
) -> Option<Event> {
    let node = declaration.into_child();
    match node.as_rule() {
        Rule::decl_accounts => {
            current.clear();
            for acc in node.into_inner() {
                current.push(parse_acc_node(ctx.accounts, acc));
            }
            None
        }
        Rule::decl_event => Some(parse_event(ctx, current.clone(), node)),
        _ => unreachable!("Unexpected rule: {:?}", node.as_rule()),
    }
}

fn parse_event(ctx: &mut Context, current: Vec<account::ID>, event: Node) -> Event {
    let mut nodes = event.into_inner();
    let schedule = parse_schedule(
        ctx,
        nodes
            .next()
            .expect("Event must have a schedule")
            .into_child(),
    );
    let statements = nodes.next().expect("Event must have a statements node");

    Event {
        schedule,
        accounts: current,
        operations: parse_statements(ctx, statements),
    }
}

fn parse_schedule(ctx: &mut Context, node: Node) -> Schedule {
    match node.as_rule() {
        Rule::time => {
            let node = node.into_child();
            match node.as_rule() {
                Rule::cron => Schedule::Cron(
                    crate::schedule::Cron::from_str(format!("0 0 12 {}", node.as_str()).as_str())
                        .expect("Pest should have validated the cron"),
                ),
                Rule::date => {
                    let seperator = node
                        .as_str()
                        .chars()
                        .nth(4)
                        .expect("Date must have a separator");
                    Schedule::Date(
                        Datestamp::parse_from_str(
                            node.as_str(),
                            &format!("%Y{}%m{}%d", seperator, seperator),
                        )
                        .unwrap(),
                    )
                }
                _ => unreachable!("Unexpected rule: {:?}", node.as_rule()),
            }
        }
        Rule::time_function => {
            let node = node.into_child();
            match node.as_rule() {
                Rule::time_func_not => {
                    Schedule::TimeFunctionNot(Box::new(parse_schedule(ctx, node.into_child())))
                }
                Rule::time_func_and => {
                    let mut nodes = node.into_inner();
                    Schedule::TimeFunctionAnd(
                        Box::new(parse_schedule(ctx, nodes.next().unwrap())),
                        Box::new(parse_schedule(ctx, nodes.next().unwrap())),
                    )
                }
                Rule::time_func_or => {
                    let mut nodes = node.into_inner();
                    let mut schedule = parse_schedule(ctx, nodes.next().unwrap());
                    for node in nodes {
                        schedule = Schedule::TimeFunctionOr(
                            Box::new(schedule),
                            Box::new(parse_schedule(ctx, node)),
                        );
                    }
                    schedule
                }
                Rule::time_func_by => {
                    let mut nodes = node.into_inner();
                    Schedule::TimeFunctionBy(
                        Box::new(parse_schedule(ctx, nodes.next().unwrap())),
                        Box::new(parse_schedule(ctx, nodes.next().unwrap())),
                    )
                }
                _ => unreachable!("Unexpected rule: {:?}", node.as_rule()),
            }
        }
        Rule::time_func_keyword => match node.as_str() {
            "today" => {
                let now = chrono::Local::now();
                Schedule::Date(now.date_naive())
            }
            "start" => Schedule::Date(ctx.date_start),
            "end" => Schedule::Date(ctx.date_end - chrono::Duration::days(1)),
            "work" => todo!("work"),
            _ => unreachable!("Unexpected rule: {:?}", node.as_rule()),
        },
        _ => unreachable!("Unexpected rule: {:?}", node.as_rule()),
    }
}

fn parse_statements(ctx: &mut Context, node: Node) -> Statements {
    match node.as_rule() {
        rule @ Rule::statements_list | rule @ Rule::statements_set => {
            let mut nodes = node.into_inner();

            let first = nodes
                .next()
                .expect("Statements list must have atleast 1 statement");
            let second = nodes.next();

            let (acc, mut stmts) = if let Some(stmts) = second {
                (
                    parse_acc_node(ctx.accounts, first),
                    stmts
                        .into_inner()
                        .map(|stmt| parse_statements(ctx, stmt))
                        .collect::<Vec<_>>(),
                )
            } else {
                (
                    ctx.accounts
                        .get_or_intern_static(if rule == Rule::statements_set {
                            "new"
                        } else {
                            "self"
                        }),
                    first
                        .into_inner()
                        .map(|stmt| parse_statements(ctx, stmt))
                        .collect(),
                )
            };

            let sym_self = ctx.accounts.get_or_intern_static("self");
            stmts.push(Statements::Single(Statement {
                from: sym_self,
                func: Box::new(move |ctx| ctx[sym_self]),
                to: ctx.accounts.get_or_intern_static("super"),
                label: None,
            }));

            if rule == Rule::statements_set {
                Statements::Set(acc, stmts)
            } else {
                Statements::List(acc, stmts)
            }
        }
        Rule::statements_single => Statements::Single(parse_statement(ctx, node.into_inner())),
        Rule::statements => parse_statements(ctx, node.into_child()),
        _ => unreachable!("Unexpected rule: {:?}", node.as_rule()),
    }
}

fn parse_statement(ctx: &mut Context, mut nodes: List) -> Statement {
    Statement {
        from: parse_acc_node(
            ctx.accounts,
            nodes.next().expect("Statement must have a from account"),
        ),
        func: parse_operation(
            ctx.accounts,
            nodes
                .next()
                .expect("Statement must have an operation")
                .into_inner(),
        ),
        to: parse_acc_node(
            ctx.accounts,
            nodes.next().expect("Statement must have a to account"),
        ),
        label: nodes.next().map(|node| node.as_str().trim().into()),
    }
}

fn parse_operation(accounts: &mut account::Interner, mut nodes: List) -> Operation {
    let first = nodes.next().expect("Operation must have atleast 1 node");
    match first.as_rule() {
        Rule::amount => {
            let amount = parse_amount(first);
            if let Some(modifier) = nodes.next() {
                parse_operation_mod(accounts, amount, modifier)
            } else {
                Box::new(move |_| amount)
            }
        }
        Rule::func => parse_operation_func(accounts, first.into_child()),
        _ => unreachable!("Unexpected rule: {:?}", first.as_rule()),
    }
}

fn parse_amount(node: Node) -> f64 {
    node.as_str()
        .replace("_", "")
        .parse()
        .expect("Amount must be a valid number")
}

fn parse_operation_mod(accounts: &mut account::Interner, amount: f64, node: Node) -> Operation {
    let node = node.into_inner().next();
    match node {
        Some(node) if node.as_rule() == Rule::rate => {
            let rate = parse_operation_rate(node);
            Box::new(move |_| amount * rate)
        }
        Some(node) if node.as_rule() == Rule::account_id => {
            let sym = parse_acc_node(accounts, node);
            Box::new(move |ctx| amount / 100.0 * ctx[sym])
        }
        None => {
            let sym_self = accounts.get_or_intern_static("self");
            Box::new(move |ctx| amount / 100.0 * ctx[sym_self])
        }
        Some(node) => unreachable!("Unexpected rule: {:?}", node.as_rule()),
    }
}

fn parse_operation_rate(node: Node) -> f64 {
    let mut nodes = node.into_inner();
    let lhs = nodes.next().expect("Rate must have a lhs");
    let rhs = nodes.next().expect("Rate must have a rhs");

    match lhs.as_str().chars().next().unwrap() {
        'y' => match rhs.as_str().chars().next().unwrap() {
            'y' => 1.0,
            'q' => 1.0 / 4.0,
            'm' => 1.0 / 12.0,
            'w' => 1.0 / 52.0,
            'd' => 1.0 / 365.0,
            _ => unreachable!("Unexpected rate: {}", rhs.as_str()),
        },
        'q' => match rhs.as_str().chars().next().unwrap() {
            'y' => 4.0,
            'q' => 1.0,
            'm' => 1.0 / 3.0,
            'w' => 1.0 / 13.0,
            'd' => 4.0 / 365.0,
            _ => unreachable!("Unexpected rate: {}", rhs.as_str()),
        },
        'm' => match rhs.as_str().chars().next().unwrap() {
            'y' => 12.0,
            'q' => 3.0,
            'm' => 1.0,
            'w' => 1.0 / 4.0,
            'd' => 1.0 / 30.0,
            _ => unreachable!("Unexpected rate: {}", rhs.as_str()),
        },
        'w' => match rhs.as_str().chars().next().unwrap() {
            'y' => 52.0,
            'q' => 13.0,
            'm' => 4.0,
            'w' => 1.0,
            'd' => 1.0 / 7.0,
            _ => unreachable!("Unexpected rate: {}", rhs.as_str()),
        },
        'd' => match rhs.as_str().chars().next().unwrap() {
            'y' => 365.0,
            'q' => 91.0,
            'm' => 30.0,
            'w' => 7.0,
            'd' => 1.0,
            _ => unreachable!("Unexpected rate: {}", rhs.as_str()),
        },
        _ => unreachable!("Unexpected rate: {}", lhs.as_str()),
    }
}

fn parse_operation_func(_accounts: &mut account::Interner, node: Node) -> Operation {
    todo!("parse_operation_func: '{}'", node.as_str())
}

pub fn compile(mut ctx: Context, source: impl AsRef<str>) -> Vec<Event> {
    use pest::Parser;
    match parser::Mercury::parse(Rule::root, source.as_ref().trim()) {
        Ok(parsed) => parse_root(&mut ctx, parsed),
        Err(e) => {
            eprintln!("Error: {}", e);
            panic!("Compile Panic! {e}");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn blah() {
        const TEST: &str = r#"
<a> (3 * *) [void > 4_000 > self: Custom label]
"#;
        use pest::Parser;
        let mut accounts = account::Interner::default();
        let mut ctx = Context {
            accounts: &mut accounts,
            date_start: Datestamp::parse_from_str("2021-01-01", "%Y-%m-%d").unwrap(),
            date_end: Datestamp::parse_from_str("2021-12-31", "%Y-%m-%d").unwrap(),
        };
        match parser::Mercury::parse(Rule::root, TEST.trim()) {
            Ok(parsed) => {
                let events = parse_root(&mut ctx, parsed);
                println!("{:#?}", events);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
