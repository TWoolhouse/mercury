use super::Datestamp;
use itertools::Itertools;

pub type Cron = cron::Schedule;

#[derive(Debug, Clone)]
pub enum Schedule {
    /// Dates that appear in the cron schema
    Cron(Cron),
    /// An exact date
    Date(Datestamp),
    /// All dates that are not in the given schedule
    TimeFunctionNot(Box<Schedule>),
    /// All dates that are in either of the given schedules
    TimeFunctionOr(Box<Schedule>, Box<Schedule>),
    /// All dates that are in both of the given schedules
    TimeFunctionAnd(Box<Schedule>, Box<Schedule>),
    /// The dates that are in the first schedule until the first date in the second schedule
    TimeFunctionBy(Box<Schedule>, Box<Schedule>),
}

impl Schedule {
    pub fn upcoming(&self, from: Datestamp) -> Box<dyn Iterator<Item = Datestamp> + '_> {
        match self {
            Schedule::Cron(cron) => Box::new(
                cron.after(
                    &from
                        .and_time(chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap())
                        .and_utc(),
                )
                .map(|d| d.date_naive()),
            ),
            Schedule::Date(date) => {
                Box::new(std::iter::once(*date).filter(move |date| date >= &from))
            }
            Schedule::TimeFunctionNot(schedule) => {
                let mut upcoming = schedule.upcoming(from).peekable();
                let mut all = from.iter_days().peekable();
                Box::new(std::iter::from_fn(move || loop {
                    if upcoming.peek() < all.peek() {
                        upcoming.next()?;
                    } else if upcoming.peek() == all.peek() {
                        upcoming.next()?;
                        all.next();
                    } else {
                        return all.next();
                    }
                }))
            }
            Schedule::TimeFunctionOr(schedule1, schedule2) => {
                Box::new(schedule1.upcoming(from).merge(schedule2.upcoming(from)))
            }
            Schedule::TimeFunctionAnd(schedule1, schedule2) => {
                Box::new(schedule1.upcoming(from).same(schedule2.upcoming(from)))
            }
            Schedule::TimeFunctionBy(schedule_predicate, schedule_next) => {
                let mut it_predicate = schedule_predicate.upcoming(from).peekable();
                let mut it_next = schedule_next.upcoming(from);
                let mut previous = None;
                Box::new(std::iter::from_fn(move || {
                    while let Some(date_next) = it_next.next() {
                        while let Some(date_predicate) = it_predicate.peek() {
                            if date_predicate >= &date_next {
                                let ret = previous;
                                previous = it_predicate.next();
                                return ret;
                            } else {
                                previous = it_predicate.next();
                            }
                        }
                    }
                    None
                }))
            }
        }
    }
}

trait Same<T>: Iterator<Item = T> {
    /// Return an iterator that yields elements that appear in both iterators.
    /// The iterators are assumed to be sorted according to the [`PartialOrd`] implementation.
    fn same(mut self, other: impl IntoIterator<Item = T>) -> impl Iterator<Item = T>
    where
        Self: Sized,
        T: PartialOrd,
    {
        let mut other = other.into_iter().peekable();
        std::iter::from_fn(move || {
            while let Some(item_self) = self.next() {
                while let Some(item_other) = other.peek() {
                    if &item_self == item_other {
                        return other.next();
                    } else if &item_self < item_other {
                        break;
                    } else {
                        other.next();
                    }
                }
            }
            None
        })
    }
}

impl<T, I> Same<I> for T
where
    T: Iterator<Item = I>,
    I: PartialOrd,
{
}

pub(crate) fn event_queue<'a: 'b, 'b, T: 'a>(
    events: impl Iterator<Item = (&'b Schedule, &'a T)>,
    from: Datestamp,
) -> impl Iterator<Item = (Datestamp, &'a T)> + 'b {
    events
        .map(move |event| event.0.upcoming(from).map(move |s| (s, event.1)))
        .kmerge_by(|a, b| a.0 < b.0)
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    fn s_cron(day: &str, month: &str) -> Schedule {
        Schedule::Cron(cron::Schedule::from_str(&format!("0 0 12 {} {} *", day, month)).unwrap())
    }

    fn s_date(string: &str) -> Schedule {
        Schedule::Date(date(string))
    }

    fn date(string: &str) -> Datestamp {
        chrono::NaiveDate::from_str(string).unwrap()
    }
    fn start() -> Datestamp {
        date("2024-01-01")
    }

    #[test]
    fn single_cron() {
        let schedule = s_cron("3", "*");
        let mut upcoming = schedule.upcoming(start());
        assert_eq!(upcoming.next().unwrap(), date("2024-01-03"));
        assert_eq!(upcoming.next().unwrap(), date("2024-02-03"));
    }

    #[test]
    fn single_date_future() {
        let schedule = s_date("2024-01-03");
        let mut upcoming = schedule.upcoming(start());
        assert_eq!(upcoming.next().unwrap(), date("2024-01-03"));
        assert_eq!(upcoming.next(), None);
    }

    #[test]
    fn single_date_past() {
        let schedule = s_date("2023-01-03");
        let mut upcoming = schedule.upcoming(start());
        assert_eq!(upcoming.next(), None);
    }

    #[test]
    fn single_or() {
        let schedule = Schedule::TimeFunctionOr(
            Box::new(s_date("2024-01-05")),
            Box::new(s_date("2024-01-03")),
        );
        let mut upcoming = schedule.upcoming(start());
        assert_eq!(upcoming.next().unwrap(), date("2024-01-03"));
        assert_eq!(upcoming.next().unwrap(), date("2024-01-05"));
    }

    #[test]
    fn single_or_cron() {
        let schedule =
            Schedule::TimeFunctionOr(Box::new(s_cron("3", "*")), Box::new(s_cron("5", "*")));
        let mut upcoming = schedule.upcoming(start());
        assert_eq!(upcoming.next().unwrap(), date("2024-01-03"));
        assert_eq!(upcoming.next().unwrap(), date("2024-01-05"));
        assert_eq!(upcoming.next().unwrap(), date("2024-02-03"));
    }

    #[test]
    fn single_and() {
        let schedule =
            Schedule::TimeFunctionAnd(Box::new(s_cron("3", "*")), Box::new(s_cron("*", "1/2")));
        let mut upcoming = schedule.upcoming(start());
        assert_eq!(upcoming.next().unwrap(), date("2024-01-03"));
        assert_eq!(upcoming.next().unwrap(), date("2024-03-03"));
        assert_eq!(upcoming.next().unwrap(), date("2024-05-03"));
    }

    #[test]
    fn single_not() {
        let schedule = Schedule::TimeFunctionAnd(
            Box::new(s_cron("3", "*")),
            Box::new(Schedule::TimeFunctionNot(Box::new(s_cron("*", "1,3")))),
        );
        let mut upcoming = schedule.upcoming(start());
        assert_eq!(upcoming.next().unwrap(), date("2024-02-03"));
        assert_eq!(upcoming.next().unwrap(), date("2024-04-03"));
    }

    #[test]
    fn events_getter() {
        let schedules = vec![
            (s_cron("8", "*"), "third"),
            (s_cron("5", "*"), "second"),
            (s_cron("3", "*"), "first"),
        ];

        let mut events = event_queue(schedules.iter().map(|e| (&e.0, e)), start());
        let mut next;

        next = events.next().unwrap();
        assert_eq!(next.0, date("2024-01-03"));
        assert_eq!(next.1 .1, "first");

        next = events.next().unwrap();
        assert_eq!(next.0, date("2024-01-05"));
        assert_eq!(next.1 .1, "second");

        next = events.next().unwrap();
        assert_eq!(next.0, date("2024-01-08"));
        assert_eq!(next.1 .1, "third");

        next = events.next().unwrap();
        assert_eq!(next.0, date("2024-02-03"));
        assert_eq!(next.1 .1, "first");
    }
}
