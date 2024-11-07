use std::{collections::HashMap, process::Output};

use crate::{
    account, process,
    transaction::{self, View},
    Datestamp, Event,
};

#[derive(Debug)]
pub struct Timeline<'e> {
    stack: account::Stack,
    interner: account::Interner,
    events: &'e [Event],
    datestamps: Vec<Datestamp>,
    history: HashMap<account::ID, (Datestamp, Vec<account::Money>)>,
}

#[derive(Debug)]
pub struct Moment<'t> {
    pub date: Datestamp,
    pub transaction: transaction::Source<'t>,
}

impl<'e> Timeline<'e> {
    pub fn new(events: &'e [Event], mut interner: account::Interner) -> Self {
        let stack = account::Stack::new(account::Symbols::new(&mut interner));
        Self {
            stack,
            interner,
            events,
            datestamps: Default::default(),
            history: Default::default(),
        }
    }

    pub fn process<'a>(
        &'a mut self,
        from: Datestamp,
        to: Datestamp,
    ) -> impl Iterator<Item = Moment<'e>> + 'a {
        Event::timeline(self.events.iter(), from.clone())
            .take_while(move |(date, _)| date < &to)
            .map(|(date, event)| {
                process::event(date, event, &mut self.stack, &mut self.interner);

                // Update the history with the current balances
                self.datestamps.push(date);
                for (acc, bal) in self.stack.balances() {
                    self.history
                        .entry(acc)
                        .or_insert_with(move || (date, Vec::with_capacity(1)))
                        .1
                        .push(bal);
                }

                Moment {
                    date: date.clone(),
                    transaction: transaction::Source {
                        date: date.clone(),
                        from: event.accounts[0],
                        to: event.accounts[0],
                        amount: 100.0,
                        label: None,
                    },
                }
            })
    }

    pub fn dates(&self) -> &[Datestamp] {
        &self.datestamps
    }

    pub fn history(&self) -> &HashMap<account::ID, (Datestamp, Vec<account::Money>)> {
        &self.history
    }
    pub fn balances(&self) -> HashMap<account::ID, account::Money> {
        self.stack.balances().collect()
    }
}

pub trait Resolve<'a, T> {
    type Output;
    fn resolve(&'a self, index: T) -> Self::Output;
}

impl<'a, 'e> Resolve<'a, account::ID> for Timeline<'e> {
    type Output = &'a str;
    fn resolve(&'a self, index: account::ID) -> Self::Output {
        self.interner.resolve(index).unwrap()
    }
}

impl<'a, 'e, 's> Resolve<'a, &'s transaction::Source<'e>> for Timeline<'e> {
    type Output = View<'a, 'e>;
    fn resolve(&'a self, index: &'s transaction::Source<'e>) -> Self::Output {
        index.as_view(&self.interner)
    }
}

impl<'a, 'e> Resolve<'a, &'a HashMap<account::ID, account::Money>> for Timeline<'e> {
    type Output = HashMap<&'a str, account::Money>;
    fn resolve(&'a self, index: &'a HashMap<account::ID, account::Money>) -> Self::Output {
        index
            .iter()
            .map(|(acc, bal)| (self.resolve(*acc), *bal))
            .collect()
    }
}

impl<'a, 'e> Resolve<'a, &'a HashMap<account::ID, (Datestamp, Vec<account::Money>)>>
    for Timeline<'e>
{
    type Output = HashMap<&'a str, (Datestamp, &'a [account::Money])>;
    fn resolve(
        &'a self,
        index: &'a HashMap<account::ID, (Datestamp, Vec<account::Money>)>,
    ) -> Self::Output {
        index
            .iter()
            .map(|(acc, (date, bal))| (self.resolve(*acc), (*date, bal.as_slice())))
            .collect()
    }
}
