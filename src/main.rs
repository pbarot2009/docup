use std::env;
use std::process;

fn main() {
    // Forward command-line arguments (excluding binary name) to cmd::run[span_1](start_span)[span_1](end_span)
    let args: Vec<String> = env::args().skip(1).collect();
    let status_code = docup::run(&args);
    process::exit(status_code);
}
