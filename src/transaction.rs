use crate::{account, Datestamp};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Source<'l> {
    pub(crate) date: Datestamp,
    pub(crate) from: account::ID,
    pub(crate) to: account::ID,
    pub(crate) amount: account::Money,
    pub(crate) label: Option<&'l str>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct View<'a, 'l> {
    pub date: Datestamp,
    pub from: &'a str,
    pub to: &'a str,
    pub amount: account::Money,
    pub label: Option<&'l str>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Record {
    pub date: Datestamp,
    pub from: String,
    pub to: String,
    pub amount: account::Money,
    pub label: Option<String>,
}

impl<'l> Source<'l> {
    pub(crate) fn as_view<'a>(&self, interner: &'a account::Interner) -> View<'a, 'l> {
        View {
            date: self.date,
            from: interner.resolve(self.from).unwrap(),
            to: interner.resolve(self.to).unwrap(),
            amount: self.amount,
            label: self.label,
        }
    }
}

impl View<'_, '_> {
    pub fn as_record(&self) -> Record {
        Record {
            date: self.date,
            from: self.from.to_string(),
            to: self.to.to_string(),
            amount: self.amount,
            label: self.label.map(|s| s.to_string()),
        }
    }
}

impl From<View<'_, '_>> for Record {
    fn from(value: View) -> Self {
        value.as_record()
    }
}

impl<'a> From<&'a Record> for View<'a, 'a> {
    fn from(value: &'a Record) -> Self {
        Self {
            date: value.date,
            from: &value.from,
            to: &value.to,
            amount: value.amount,
            label: value.label.as_deref(),
        }
    }
}
