fn main() {
    let store = mercury::parse(include_str!("test.mcy").trim());
    match store {
        Ok(store) => {
            let mut values = mercury::evaluate(&store);
            values.sort_by_key(|x| x.0);
            println!("{:?}", values);
        }
        Err(e) => println!("{}", e),
    }
}
