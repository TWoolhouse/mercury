use std::{env, fs};

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

    let store = mercury::parse(file_contents.trim());
    match store {
        Ok(store) => {
            let mut values = mercury::evaluate(&store);
            values.sort_by_key(|x| x.0);
            println!("{:?}", values);
        }
        Err(e) => eprintln!("{}", e),
    }
}
