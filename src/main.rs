use std::env;
use std::process;

fn main() {
    // Forward command-line arguments (excluding binary name) to cmd::run
    let args: Vec<String> = env::args().skip(1).collect();
    let status_code = docup::run(&args);
    process::exit(status_code);
}
