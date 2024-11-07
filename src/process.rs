use crate::{account, Datestamp, Event, Statements};

pub(crate) fn event<'e, 'i>(
    _date: Datestamp,
    event: &'e Event,
    stack: &mut account::Stack,
    interner: &'i mut account::Interner,
) {
    for account in &event.accounts {
        stack.push(*account);
        statements(&event.operations, stack, interner);
        stack.pop();
    }
}

pub(crate) fn statements(
    stmts: &Statements,
    stack: &mut account::Stack,
    interner: &mut account::Interner,
) {
    match stmts {
        Statements::List(acid, list) => {
            stack.push(*acid);
            for stmt in list {
                statements(stmt, stack, interner);
            }
            stack.pop();
        }
        Statements::Set(acid, set) => {
            stack.push(*acid);
            let mut shadows = stack.split(set.len());
            for (shadow, stmt) in shadows.iter_mut().zip(set.iter()) {
                statements(stmt, shadow, interner);
            }
            stack.merge(shadows.into_iter());
            stack.pop();
        }
        Statements::Single(stmt) => {
            let delta = (stmt.func)(&mut account::CtxMut::new(stack, interner));
            stack[stmt.from] -= delta;
            stack[stmt.to] += delta;
        }
    }
}
