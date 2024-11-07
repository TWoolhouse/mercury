use mercury::Resolve;
use std::{collections::HashMap, env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Please provide a file path as a command line argument");
        return;
    }
    let file_path = &args[1];

    // Read the file contents
    let file_contents = match fs::read_to_string(file_path) {
        Ok(contents) => contents,
        Err(error) => {
            eprintln!("Error reading file: {}", error);
            return;
        }
    };

    let mut accounts = mercury::account::Interner::default();

    let from = chrono::Local::now().date_naive();
    let to = chrono::Local::now()
        .date_naive()
        .checked_add_days(chrono::Days::new(365))
        .unwrap();

    let events = mercury::syntax::compile(
        mercury::syntax::Context {
            accounts: &mut accounts,
            date_start: from,
            date_end: to,
        },
        file_contents.as_str(),
    );
    let mut timeline = mercury::Timeline::new(&events, accounts);

    {
        timeline.process(from, to).for_each(drop);
    }
    let history = timeline.resolve(timeline.history());
    let dates = timeline.dates();
    let full_history = history
        .into_iter()
        .map(|(acc, (_, balances))| {
            (acc, {
                let mut out = vec![0.0; dates.len() - balances.len()];
                out.reserve_exact(balances.len());
                out.extend(balances);
                out
            })
        })
        .collect::<HashMap<_, _>>();

    for (acc, balances) in full_history {
        println!("{}: {:?}", acc, balances);
    }
    println!("Dates: {:?}", dates);
}
