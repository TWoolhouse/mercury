use mercury::Resolve;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn initialise() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen(getter_with_clone)]
pub struct Account {
    pub name: String,
    pub history: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen(getter_with_clone)]
pub struct Output {
    pub dates: Vec<String>,
    pub accounts: Vec<Account>,
}

fn parse_from_until(
    input: &str,
    from: mercury::Datestamp,
    to: mercury::Datestamp,
) -> (
    Vec<mercury::Datestamp>,
    Vec<(String, Vec<mercury::account::Money>)>,
) {
    let mut accounts = mercury::account::Interner::default();

    let events = mercury::syntax::compile(
        mercury::syntax::Context {
            accounts: &mut accounts,
            date_start: from,
            date_end: to,
        },
        input,
    );

    let mut timeline = mercury::Timeline::new(&events, accounts);
    timeline.process(from, to).for_each(drop);

    let history = timeline.resolve(timeline.history());
    let dates = timeline.dates();
    let full_history = history
        .into_iter()
        .map(|(acc, (_, balances))| {
            (acc.to_owned(), {
                let mut out = vec![0.0; dates.len() - balances.len()];
                out.reserve_exact(balances.len());
                out.extend(balances);
                out
            })
        })
        .collect::<Vec<_>>();

    (dates.into(), full_history)
}

#[wasm_bindgen]
pub fn parse(input: &str, from: &str, to: &str) -> Result<Output, String> {
    const DATE_FORMAT: &str = "%Y-%m-%d";

    let from = mercury::Datestamp::parse_from_str(from, DATE_FORMAT).map_err(|e| e.to_string())?;
    let to = mercury::Datestamp::parse_from_str(to, DATE_FORMAT).map_err(|e| e.to_string())?;
    let (dates, accounts) = parse_from_until(input, from, to);

    Ok(Output {
        dates: dates
            .into_iter()
            .map(|d| d.format(DATE_FORMAT).to_string())
            .collect(),
        accounts: accounts
            .into_iter()
            .map(|(name, history)| Account { name, history })
            .collect(),
    })
}
