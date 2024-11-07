use std::collections::HashMap;

pub type ID = string_interner::DefaultSymbol;
pub type Money = f64;

pub type Interner = string_interner::DefaultStringInterner;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Symbols {
    new_: ID,
    self_: ID,
    super_: ID,
}

impl Symbols {
    pub(crate) fn new(interner: &mut Interner) -> Self {
        Self {
            new_: interner.get_or_intern_static("new"),
            self_: interner.get_or_intern_static("self"),
            super_: interner.get_or_intern_static("super"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
enum StackID {
    Layer(u32),
    Account(ID),
}

impl From<ID> for StackID {
    fn from(account: ID) -> Self {
        StackID::Account(account)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Stack {
    symbols: Symbols,
    balances: HashMap<StackID, Money>,
    children: Vec<StackID>,
}

impl Stack {
    pub(crate) fn new(symbols: Symbols) -> Self {
        Self {
            symbols,
            balances: Default::default(),
            children: Default::default(),
        }
    }

    pub(crate) fn push(&mut self, account: ID) {
        self.children.push(self.resolve(account));
    }

    pub(crate) fn pop(&mut self) {
        self.children.pop();
    }

    fn resolve(&self, account: ID) -> StackID {
        if account == self.symbols.super_ {
            self.children[self.children.len() - 2]
        } else if account == self.symbols.self_ {
            self.children[self.children.len() - 1]
        } else if account == self.symbols.new_ {
            StackID::Layer(self.children.len() as u32)
        } else {
            account.into()
        }
    }

    pub(crate) fn split(&self, count: usize) -> Vec<Self> {
        vec![self.clone(); count]
    }

    pub(crate) fn merge(&mut self, stacks: impl Iterator<Item = Self>) {
        let mut deltas: HashMap<StackID, f64> = HashMap::default();
        for stack in stacks {
            for (acid, balance) in stack.balances.into_iter() {
                *deltas.entry(acid).or_insert(0.0) +=
                    balance - self.balances.get(&acid).unwrap_or(&0.0);
            }
        }
        for (acid, delta) in deltas.into_iter() {
            *self.balances.entry(acid).or_insert(0.0) += delta;
        }
    }

    pub(crate) fn balances(&self) -> impl Iterator<Item = (ID, Money)> + '_ {
        self.balances
            .iter()
            .filter_map(|(acid, balance)| match acid {
                StackID::Account(id) => Some((*id, *balance)),
                _ => None,
            })
    }
}

impl From<Stack> for HashMap<ID, Money> {
    fn from(stack: Stack) -> Self {
        stack
            .balances
            .into_iter()
            .filter_map(|(acid, balance)| match acid {
                StackID::Account(id) => Some((id, balance)),
                _ => None,
            })
            .collect()
    }
}

impl std::ops::Index<StackID> for Stack {
    type Output = Money;

    fn index(&self, index: StackID) -> &Self::Output {
        self.balances.get(&index).unwrap_or(&0.0)
    }
}

impl std::ops::Index<ID> for Stack {
    type Output = Money;

    fn index(&self, index: ID) -> &Self::Output {
        self.index(self.resolve(index))
    }
}

impl std::ops::IndexMut<StackID> for Stack {
    fn index_mut(&mut self, index: StackID) -> &mut Self::Output {
        self.balances.entry(index).or_insert(0.0)
    }
}

impl std::ops::IndexMut<ID> for Stack {
    fn index_mut(&mut self, index: ID) -> &mut Self::Output {
        self.index_mut(self.resolve(index))
    }
}

pub struct CtxMut<'s, 'i> {
    stack: &'s mut Stack,
    interner: &'i mut Interner,
}

impl<'s, 'i> CtxMut<'s, 'i> {
    pub(crate) fn new(stack: &'s mut Stack, interner: &'i mut Interner) -> Self {
        Self { stack, interner }
    }
}

impl CtxMut<'_, '_> {
    pub(crate) fn register(&mut self, account: &str) -> ID {
        self.interner.get_or_intern(account)
    }
}

impl std::ops::Index<ID> for CtxMut<'_, '_> {
    type Output = Money;

    fn index(&self, index: ID) -> &Self::Output {
        &self.stack[index]
    }
}

impl std::ops::IndexMut<ID> for CtxMut<'_, '_> {
    fn index_mut(&mut self, index: ID) -> &mut Self::Output {
        &mut self.stack[index]
    }
}

impl std::ops::Index<&str> for CtxMut<'_, '_> {
    type Output = Money;

    fn index(&self, index: &str) -> &Self::Output {
        &self.stack[self.interner.get(index).expect("Account not found")]
    }
}

impl std::ops::IndexMut<&str> for CtxMut<'_, '_> {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        &mut self.stack[self.interner.get_or_intern(index)]
    }
}
