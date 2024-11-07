use crate::{account, Datestamp, Schedule, Statements};

#[derive(Debug)]
pub struct Event {
    pub schedule: Schedule,
    pub accounts: Vec<account::ID>,
    pub operations: Statements,
}

impl Event {
    pub fn timeline<'a>(
        events: impl Iterator<Item = &'a Event>,
        from: Datestamp,
    ) -> impl Iterator<Item = (Datestamp, &'a Event)> {
        crate::schedule::event_queue(events.map(|e| (&e.schedule, e)), from)
    }
}

#[derive(Debug)]
pub struct TransactionView<'a, 'e> {
    date: Datestamp,
    from: account::ID,
    to: account::ID,
    amount: f64,
    description: Option<&'e str>,
    interner: &'a crate::account::Interner,
}

impl<'a, 'e> TransactionView<'a, 'e> {
    fn from(&self) -> &str {
        self.interner.resolve(self.from).unwrap()
    }

    fn to(&self) -> &str {
        self.interner.resolve(self.to).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub date: Datestamp,
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub description: Option<String>,
}

impl<'a, 'e> From<TransactionView<'a, 'e>> for Transaction {
    fn from(view: TransactionView<'a, 'e>) -> Self {
        Self {
            date: view.date,
            from: view.from().to_string(),
            to: view.to().to_string(),
            amount: view.amount,
            description: view.description.map(|s| s.to_string()),
        }
    }
}
