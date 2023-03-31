use std::{env, process};

mod args;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (cmd, args) = match args::parse_args(args) {
        Ok(args) => args,
        Err(e) => {
            println!("Error: {}", e);
            process::exit(1);
        }
    };

    dbg!(cmd, args);
}
