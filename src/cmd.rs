use std::io::IsTerminal;
use std::path::Path;
use std::time::Instant;

use crate::codegen::generate;
use crate::errors::{PositionedError, source_snippet};
use crate::parser::Parser;
use crate::sema::analyze;

/// docup compiler version reported by `docup version` and `docup --version`[span_1](start_span)[span_1](end_span).
pub const VERSION: &str = "0.2.0";

#[derive(Clone, Copy)]
pub struct Colors {
    pub reset: &'static str,
    pub red: &'static str,
    pub green: &'static str,
    pub yellow: &'static str,
    pub blue: &'static str,
    pub cyan: &'static str,
    pub bold: &'static str,
    pub dim: &'static str,
}

impl Colors {
    pub fn detect() -> Self {
        // Disable ANSI color if NO_COLOR is set or if stdout is not a terminal[span_2](start_span)[span_2](end_span).
        let enabled = std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal();
        if enabled {
            Self {
                reset: "\x1b[0m",
                red: "\x1b[31m",
                green: "\x1b[32m",
                yellow: "\x1b[33m",
                blue: "\x1b[34m",
                cyan: "\x1b[36m",
                bold: "\x1b[1m",
                dim: "\x1b[2m",
            }
        } else {
            Self {
                reset: "",
                red: "",
                green: "",
                yellow: "",
                blue: "",
                cyan: "",
                bold: "",
                dim: "",
            }
        }
    }
}

/// Options holds parsed build flags[span_3](start_span)[span_3](end_span).
#[derive(Debug, Default, Clone)]
pub struct Options {
    pub input_path: String,
    pub output_path: String,
    pub verbose: bool,
    pub quiet: bool,
}

/// Run is the main CLI entry point called by `main`[span_4](start_span)[span_4](end_span).
pub fn run(args: &[String]) -> i32 {
    let colors = Colors::detect();

    if args.is_empty() {
        print_usage(colors);
        return 1;
    }

    match args[0].as_str() {
        "build" => run_build(colors, &args[1..]),
        "version" | "-v" | "--version" => {
            println!("docup version {VERSION}");
            0
        }
        "help" | "-h" | "--help" => {
            print_usage(colors);
            0
        }
        other => {
            fail(colors, &format!("unknown command \"{other}\""));
            println!();
            print_usage(colors);
            1
        }
    }
}

pub fn print_usage(c: Colors) {
    println!(
        "{}{}DocUP Compiler{} {}— compiles .du documents to standalone HTML5{}\n",
        c.bold, c.cyan, c.reset, c.dim, c.reset
    );
    println!("{}USAGE{}", c.bold, c.reset);
    println!("  docup build <input.du> [flags]");
    println!("  docup version");
    println!("  docup help\n");
    println!("{}FLAGS{}", c.bold, c.reset);
    println!("  -o <file>       output HTML path (default: input with .html extension)");
    println!("  --verbose, -V   print detailed timing for each build stage");
    println!("  --quiet, -q     suppress step-by-step progress output\n");
    println!("{}EXAMPLES{}", c.bold, c.reset);
    println!("  docup build report.du");
    println!("  docup build report.du -o dist/report.html");
    println!("  docup build report.du --quiet");
}

fn run_build(c: Colors, args: &[String]) -> i32 {
    let opts = match parse_build_args(c, args) {
        Some(o) => o,
        None => return 1,
    };

    if opts.input_path.is_empty() {
        fail(c, "no input .du file specified");
        println!();
        print_usage(c);
        return 1;
    }

    let p = Path::new(&opts.input_path);
    let is_du = p
        .extension()
        .map(|ext| ext.eq_ignore_ascii_case("du"))
        .unwrap_or(false);
    if !is_du {
        warn(
            c,
            &format!(
                "input file {:?} does not have a .du extension",
                opts.input_path
            ),
        );
    }

    let mut final_opts = opts;
    if final_opts.output_path.is_empty() {
        final_opts.output_path = swap_ext(&final_opts.input_path, ".html");
    }

    build(c, final_opts)
}

fn parse_build_args(c: Colors, args: &[String]) -> Option<Options> {
    let mut opts = Options::default();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-o" | "--output" => {
                if i + 1 >= args.len() || looks_like_flag(&args[i + 1]) {
                    fail(c, &format!("missing value for {arg}"));
                    return None;
                }
                opts.output_path = args[i + 1].clone();
                i += 1;
            }
            "--verbose" | "-V" => {
                opts.verbose = true;
            }
            "--quiet" | "-q" => {
                opts.quiet = true;
            }
            _ => {
                if looks_like_flag(arg) {
                    fail(c, &format!("unknown flag \"{arg}\""));
                    return None;
                }
                if !opts.input_path.is_empty() {
                    fail(
                        c,
                        &format!(
                            "unexpected extra argument \"{arg}\" (input file already set to \"{}\")",
                            opts.input_path
                        ),
                    );
                    return None;
                }
                opts.input_path = arg.clone();
            }
        }
        i += 1;
    }

    if opts.verbose && opts.quiet {
        fail(c, "--verbose and --quiet cannot be used together");
        return None;
    }

    Some(opts)
}

fn looks_like_flag(s: &str) -> bool {
    s.starts_with('-') && s != "-"
}

fn build(c: Colors, opts: Options) -> i32 {
    let overall_start = Instant::now();
    let mut stage_start = Instant::now();
    let total_stages = 5;

    let mut log_stage = |n: usize, verb: &str, detail: &str| {
        if opts.quiet {
            return;
        }
        if opts.verbose && n > 1 {
            println!("{}      ({:.2}ms){}", c.dim, ms_since(stage_start), c.reset);
        }
        stage_start = Instant::now();
        step(c, n, total_stages, verb, detail);
    };

    // Stage 1: Reading input
    log_stage(1, "Reading", &opts.input_path);
    let source = match std::fs::read(&opts.input_path) {
        Ok(bytes) => bytes,
        Err(err) => {
            report_file_error(c, &opts.input_path, &err);
            return 1;
        }
    };
    if source.iter().all(|b| b.is_ascii_whitespace()) {
        fail(c, &format!("{} is empty", opts.input_path));
        return 1;
    }

    // Stage 2: Lexing & parsing
    log_stage(2, "Lexing & parsing", &opts.input_path);
    let mut parser = match Parser::new(&source) {
        Ok(p) => p,
        Err(err) => {
            report_compile_error(c, &opts.input_path, &source, &err);
            return 1;
        }
    };
    let doc = match parser.parse_document() {
        Ok(d) => d,
        Err(err) => {
            report_compile_error(c, &opts.input_path, &source, &err);
            return 1;
        }
    };

    // Stage 3: Semantic analysis
    log_stage(3, "Analyzing", "semantic checks");
    if let Err(err) = analyze(&doc) {
        report_compile_error(c, &opts.input_path, &source, &err);
        return 1;
    }

    // Stage 4: Codegen
    log_stage(4, "Generating", "HTML5 output");
    let out = generate(&doc);

    // Stage 5: Writing output
    log_stage(5, "Writing", &opts.output_path);
    if let Err(err) = std::fs::write(&opts.output_path, out.as_bytes()) {
        report_file_error(c, &opts.output_path, &err);
        return 1;
    }

    if opts.verbose && !opts.quiet {
        println!("{}      ({:.2}ms){}", c.dim, ms_since(stage_start), c.reset);
    }

    if !opts.quiet {
        let elapsed = overall_start.elapsed();
        let size = out.len();
        println!(
            "\n{}{}✓ build succeeded{} in {}{:.2}ms{}",
            c.bold,
            c.green,
            c.reset,
            c.cyan,
            elapsed.as_micros() as f64 / 1000.0,
            c.reset
        );
        println!(
            "  {}→{} {}{}{} {}({}){}",
            c.green,
            c.reset,
            c.bold,
            opts.output_path,
            c.reset,
            c.dim,
            human_size(size),
            c.reset
        );
    }

    0
}

fn ms_since(t: Instant) -> f64 {
    t.elapsed().as_micros() as f64 / 1000.0
}

fn human_size(bytes: usize) -> String {
    const UNIT: usize = 1024;
    if bytes < UNIT {
        return format!("{bytes} B");
    }
    let mut div = UNIT;
    let mut exp = 0;
    let units = ['K', 'M', 'G', 'T', 'P', 'E'];
    let mut n = bytes / UNIT;
    while n >= UNIT && exp < units.len() - 1 {
        div *= UNIT;
        exp += 1;
        n /= UNIT;
    }
    format!("{:.1} {}B", bytes as f64 / div as f64, units[exp])
}

fn step(c: Colors, n: usize, total: usize, verb: &str, detail: &str) {
    println!(
        "{}[{n}/{total}]{} {}{}{} {detail}",
        c.blue, c.reset, c.bold, verb, c.reset
    );
}

fn fail(c: Colors, msg: &str) {
    eprintln!("{}✗ error:{} {msg}", c.red, c.reset);
}

fn warn(c: Colors, msg: &str) {
    eprintln!("{}⚠ warning:{} {msg}", c.yellow, c.reset);
}

fn report_file_error(c: Colors, path: &str, err: &std::io::Error) {
    match err.kind() {
        std::io::ErrorKind::NotFound => fail(c, &format!("file not found: {path}")),
        std::io::ErrorKind::PermissionDenied => fail(c, &format!("permission denied: {path}")),
        _ => fail(c, &format!("cannot access {path}: {err}")),
    }
}

fn report_compile_error(
    c: Colors,
    path: &str,
    source: &[u8],
    err: &(dyn PositionedError + 'static),
) {
    let (line, col) = err.position();
    eprintln!("{}✗ error:{} {err}", c.red, c.reset);
    eprintln!("  {}--> {path}:{line}:{col}{}", c.cyan, c.reset);
    let snippet = source_snippet(source, line, col);
    if !snippet.is_empty() {
        for l in snippet.lines() {
            eprintln!("  {}{l}{}", c.yellow, c.reset);
        }
    }
}

fn swap_ext(path: &str, new_ext: &str) -> String {
    let p = Path::new(path);
    if let Some(ext) = p.extension() {
        let ext_str = ext.to_string_lossy();
        let cut = path.len() - ext_str.len() - 1;
        format!("{}{}", &path[..cut], new_ext)
    } else {
        format!("{path}{new_ext}")
    }
}
