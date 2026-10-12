#![forbid(unsafe_code)]

use std::env;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();
    match lpdf_cli::run_cli(&args) {
        Ok(msg) => {
            if !msg.is_empty() {
                println!("{}", msg);
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            exit(1);
        }
    }
}
