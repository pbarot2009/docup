use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, IsTerminal, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use crate::ast::{BlockNode, DocumentNode};
use crate::codegen::generate;
use crate::errors::{source_snippet, PositionedError, SemaError};
use crate::parser::Parser;
use crate::sema::analyze;

/// docup compiler version reported by `docup version` and `docup --version`.
pub const VERSION: &str = "0.2.0";

const LIVE_RELOAD_SCRIPT: &str = r#"  <script>
    (function() {
      const es = new EventSource('/docup-events');
      es.onmessage = function(e) {
        if (e.data === 'reload') window.location.reload();
      };
    })();
  </script>
"#;

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

/// Options holds parsed flags for build and watch commands.
#[derive(Debug, Clone)]
pub struct Options {
    pub input_path: String,
    pub output_path: String,
    pub style_path: Option<String>,
    pub port: u16,
    pub verbose: bool,
    pub quiet: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            input_path: String::new(),
            output_path: String::new(),
            style_path: None,
            port: 8080,
            verbose: false,
            quiet: false,
        }
    }
}

pub struct BuildResult {
    pub html: String,
    pub watched_files: HashSet<PathBuf>,
}

struct IncludeError {
    path: String,
    source: Vec<u8>,
    error: SemaError,
}

/// Run is the main CLI entry point called by `main`.
pub fn run(args: &[String]) -> i32 {
    let colors = Colors::detect();

    if args.is_empty() {
        print_usage(colors);
        return 1;
    }

    match args[0].as_str() {
        "build" => run_build(colors, &args[1..]),
        "watch" => run_watch(colors, &args[1..]),
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
    println!("  docup watch <input.du> [flags]");
    println!("  docup version");
    println!("  docup help\n");
    println!("{}FLAGS{}", c.bold, c.reset);
    println!("  -o, --output <file>  output HTML path (default: input with .html extension)");
    println!("  -s, --style <file>   custom CSS stylesheet to inject into output HTML");
    println!("  -p, --port <port>    local development server port for watch mode (default: 8080)");
    println!("  --verbose, -V        print detailed timing for each build stage");
    println!("  --quiet, -q          suppress step-by-step progress output\n");
    println!("{}EXAMPLES{}", c.bold, c.reset);
    println!("  docup build report.du");
    println!("  docup build report.du -o dist/index.html -s style.css");
    println!("  docup watch report.du --port 3000");
}

fn run_build(c: Colors, args: &[String]) -> i32 {
    let opts = match parse_cli_args(c, args) {
        Some(o) => o,
        None => return 1,
    };

    if opts.input_path.is_empty() {
        fail(c, "no input .du file specified");
        println!();
        print_usage(c);
        return 1;
    }

    validate_extension(c, &opts.input_path);

    let mut final_opts = opts;
    if final_opts.output_path.is_empty() {
        final_opts.output_path = swap_ext(&final_opts.input_path, ".html");
    }

    match compile_pipeline(c, &final_opts) {
        Ok(res) => {
            if let Err(err) = std::fs::write(&final_opts.output_path, res.html.as_bytes()) {
                report_file_error(c, &final_opts.output_path, &err);
                return 1;
            }
            0
        }
        Err(_) => 1,
    }
}

fn run_watch(c: Colors, args: &[String]) -> i32 {
    let opts = match parse_cli_args(c, args) {
        Some(o) => o,
        None => return 1,
    };

    if opts.input_path.is_empty() {
        fail(c, "no input .du file specified");
        println!();
        print_usage(c);
        return 1;
    }

    validate_extension(c, &opts.input_path);

    let mut final_opts = opts;
    if final_opts.output_path.is_empty() {
        final_opts.output_path = swap_ext(&final_opts.input_path, ".html");
    }

    let initial_build = match compile_pipeline(c, &final_opts) {
        Ok(res) => {
            let _ = std::fs::write(&final_opts.output_path, res.html.as_bytes());
            res
        }
        Err(_) => BuildResult {
            html: "<h1>Compilation Error</h1><p>Check terminal output for details.</p>".to_string(),
            watched_files: {
                let mut s = HashSet::new();
                s.insert(PathBuf::from(&final_opts.input_path));
                s
            },
        },
    };

    let shared_html = Arc::new(Mutex::new(inject_live_reload(&initial_build.html)));
    let sse_clients: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));

    let server_addr = format!("127.0.0.1:{}", final_opts.port);
    let listener = match TcpListener::bind(&server_addr) {
        Ok(l) => l,
        Err(err) => {
            fail(
                c,
                &format!("failed to bind dev server to {server_addr}: {err}"),
            );
            return 1;
        }
    };

    let base_dir = Path::new(&final_opts.input_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    // Spawn embedded local HTTP server thread
    let server_html = Arc::clone(&shared_html);
    let server_sse = Arc::clone(&sse_clients);
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let html_ref = Arc::clone(&server_html);
            let sse_ref = Arc::clone(&server_sse);
            let dir_ref = base_dir.clone();
            thread::spawn(move || {
                handle_http_client(stream, &dir_ref, &html_ref, &sse_ref);
            });
        }
    });

    println!(
        "\n{}{}⚡ DocUP dev server running at{} {}http://{server_addr}{}",
        c.bold, c.green, c.reset, c.cyan, c.reset
    );
    println!(
        "{}   Watching for file changes... (Press Ctrl+C to stop){}\n",
        c.dim, c.reset
    );

    let mut watched_files = initial_build.watched_files;
    if let Some(ref style) = final_opts.style_path {
        watched_files.insert(PathBuf::from(style));
    }
    let mut file_mtimes = snapshot_mtimes(&watched_files);

    loop {
        thread::sleep(Duration::from_millis(250));

        let current_mtimes = snapshot_mtimes(&watched_files);
        let changed = current_mtimes
            .iter()
            .any(|(path, mtime)| file_mtimes.get(path) != Some(mtime));

        if changed {
            file_mtimes = current_mtimes;
            println!("{}↻ Change detected, recompiling...{}", c.cyan, c.reset);

            match compile_pipeline(c, &final_opts) {
                Ok(new_build) => {
                    let _ = std::fs::write(&final_opts.output_path, new_build.html.as_bytes());

                    {
                        let mut h = shared_html.lock().unwrap();
                        *h = inject_live_reload(&new_build.html);
                    }

                    watched_files = new_build.watched_files;
                    if let Some(ref style) = final_opts.style_path {
                        watched_files.insert(PathBuf::from(style));
                    }
                    file_mtimes = snapshot_mtimes(&watched_files);

                    notify_reload(&sse_clients);
                }
                Err(_) => {
                    eprintln!("{}✗ Build failed; waiting for fix...{}", c.yellow, c.reset);
                }
            }
        }
    }
}

fn inject_live_reload(html: &str) -> String {
    if let Some(pos) = html.rfind("</body>") {
        let mut with_script = html[..pos].to_string();
        with_script.push_str(LIVE_RELOAD_SCRIPT);
        with_script.push_str(&html[pos..]);
        with_script
    } else {
        format!("{html}{LIVE_RELOAD_SCRIPT}")
    }
}

fn notify_reload(sse_clients: &Arc<Mutex<Vec<TcpStream>>>) {
    let mut clients = sse_clients.lock().unwrap();
    clients.retain_mut(|client| {
        let _ = client.set_write_timeout(Some(Duration::from_millis(500)));
        client.write_all(b"data: reload\n\n").is_ok()
    });
}

fn handle_http_client(
    mut stream: TcpStream,
    base_dir: &Path,
    current_html: &Arc<Mutex<String>>,
    sse_clients: &Arc<Mutex<Vec<TcpStream>>>,
) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let mut reader = BufReader::new(&mut stream);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() || request_line.is_empty() {
        return;
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 || parts[0] != "GET" {
        let _ = stream.write_all(b"HTTP/1.1 405 Method Not Allowed\r\n\r\n");
        return;
    }

    let path = parts[1].split('?').next().unwrap_or("/");

    if path == "/docup-events" {
        let response_headers = "HTTP/1.1 200 OK\r\n\
            Content-Type: text/event-stream\r\n\
            Cache-Control: no-cache\r\n\
            Connection: keep-alive\r\n\
            Access-Control-Allow-Origin: *\r\n\r\n";

        if stream.write_all(response_headers.as_bytes()).is_ok() {
            let _ = stream.set_write_timeout(None);
            let mut clients = sse_clients.lock().unwrap();
            clients.push(stream);
        }
        return;
    }

    if path == "/" || path == "/index.html" {
        let body = {
            let h = current_html.lock().unwrap();
            h.clone()
        };
        let response = format!(
            "HTTP/1.1 200 OK\r\n\
            Content-Type: text/html; charset=utf-8\r\n\
            Content-Length: {}\r\n\
            Connection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    let clean_path = path.trim_start_matches('/');
    let asset_path = base_dir.join(clean_path);

    if asset_path.is_file() {
        if let Ok(data) = std::fs::read(&asset_path) {
            let mime = mime_for_path(&asset_path);
            let header = format!(
                "HTTP/1.1 200 OK\r\n\
                Content-Type: {mime}\r\n\
                Content-Length: {}\r\n\
                Connection: close\r\n\r\n",
                data.len()
            );
            let _ = stream.write_all(header.as_bytes());
            let _ = stream.write_all(&data);
            return;
        }
    }

    let not_found =
        "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found";
    let _ = stream.write_all(not_found.as_bytes());
}

fn mime_for_path(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        _ => "application/octet-stream",
    }
}

fn snapshot_mtimes(paths: &HashSet<PathBuf>) -> HashMap<PathBuf, Option<SystemTime>> {
    paths
        .iter()
        .map(|p| {
            let mtime = std::fs::metadata(p).and_then(|m| m.modified()).ok();
            (p.clone(), mtime)
        })
        .collect()
}

fn compile_pipeline(c: Colors, opts: &Options) -> Result<BuildResult, ()> {
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
            return Err(());
        }
    };
    if source.iter().all(|b| b.is_ascii_whitespace()) {
        fail(c, &format!("{} is empty", opts.input_path));
        return Err(());
    }

    // Stage 2: Lexing & parsing
    log_stage(2, "Lexing & parsing", &opts.input_path);
    let mut parser = match Parser::new(&source) {
        Ok(p) => p,
        Err(err) => {
            report_compile_error(c, &opts.input_path, &source, &err);
            return Err(());
        }
    };
    let mut doc = match parser.parse_document() {
        Ok(d) => d,
        Err(err) => {
            report_compile_error(c, &opts.input_path, &source, &err);
            return Err(());
        }
    };

    // Resolve modular includes
    let root_path = Path::new(&opts.input_path);
    let canonical_root = root_path
        .canonicalize()
        .unwrap_or_else(|_| root_path.to_path_buf());
    let mut visited = HashSet::new();
    visited.insert(canonical_root.clone());
    let mut watched_files = HashSet::new();
    watched_files.insert(canonical_root);

    if let Err(inc_err) = resolve_document_includes(
        root_path,
        &source,
        &mut doc,
        &mut visited,
        &mut watched_files,
    ) {
        report_compile_error(c, &inc_err.path, &inc_err.source, &inc_err.error);
        return Err(());
    }

    // Stage 3: Semantic analysis
    log_stage(3, "Analyzing", "semantic checks");
    if let Err(err) = analyze(&doc) {
        report_compile_error(c, &opts.input_path, &source, &err);
        return Err(());
    }

    // Stage 4: Codegen
    let custom_css = match &opts.style_path {
        Some(path) => match std::fs::read_to_string(path) {
            Ok(content) => Some(content),
            Err(err) => {
                report_file_error(c, path, &err);
                return Err(());
            }
        },
        None => None,
    };

    log_stage(4, "Generating", "HTML5 output");
    let out = generate(&doc, custom_css.as_deref());

    // Stage 5: Target preparation & report
    log_stage(5, "Writing", &opts.output_path);

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

    Ok(BuildResult {
        html: out,
        watched_files,
    })
}

fn resolve_document_includes(
    file_path: &Path,
    source: &[u8],
    doc: &mut DocumentNode,
    visited: &mut HashSet<PathBuf>,
    watched: &mut HashSet<PathBuf>,
) -> Result<(), IncludeError> {
    let mut new_blocks = Vec::with_capacity(doc.blocks.len());

    for block in doc.blocks.drain(..) {
        if let BlockNode::Include(inc) = block {
            let base_dir = file_path.parent().unwrap_or_else(|| Path::new("."));
            let target_path = base_dir.join(&inc.path);
            let canonical = match target_path.canonicalize() {
                Ok(c) => c,
                Err(err) => {
                    return Err(IncludeError {
                        path: file_path.to_string_lossy().into_owned(),
                        source: source.to_vec(),
                        error: SemaError::new(
                            inc.line,
                            inc.col,
                            format!("cannot resolve include path \"{}\": {err}", inc.path),
                        ),
                    });
                }
            };

            if visited.contains(&canonical) {
                return Err(IncludeError {
                    path: file_path.to_string_lossy().into_owned(),
                    source: source.to_vec(),
                    error: SemaError::new(
                        inc.line,
                        inc.col,
                        format!("circular include detected for \"{}\"", inc.path),
                    ),
                });
            }

            visited.insert(canonical.clone());
            watched.insert(canonical.clone());

            let child_source = match std::fs::read(&canonical) {
                Ok(bytes) => bytes,
                Err(err) => {
                    return Err(IncludeError {
                        path: file_path.to_string_lossy().into_owned(),
                        source: source.to_vec(),
                        error: SemaError::new(
                            inc.line,
                            inc.col,
                            format!("cannot read include file \"{}\": {err}", inc.path),
                        ),
                    });
                }
            };

            let mut child_parser = match Parser::new(&child_source) {
                Ok(p) => p,
                Err(err) => {
                    return Err(IncludeError {
                        path: canonical.to_string_lossy().into_owned(),
                        source: child_source,
                        error: SemaError::new(err.line, err.col, err.message),
                    });
                }
            };

            let mut child_doc = match child_parser.parse_document() {
                Ok(d) => d,
                Err(err) => {
                    return Err(IncludeError {
                        path: canonical.to_string_lossy().into_owned(),
                        source: child_source,
                        error: SemaError::new(err.line, err.col, err.message),
                    });
                }
            };

            resolve_document_includes(&canonical, &child_source, &mut child_doc, visited, watched)?;
            visited.remove(&canonical);

            new_blocks.extend(child_doc.blocks);
        } else {
            new_blocks.push(block);
        }
    }

    doc.blocks = new_blocks;
    Ok(())
}

fn parse_cli_args(c: Colors, args: &[String]) -> Option<Options> {
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
            "-s" | "--style" => {
                if i + 1 >= args.len() || looks_like_flag(&args[i + 1]) {
                    fail(c, &format!("missing value for {arg}"));
                    return None;
                }
                opts.style_path = Some(args[i + 1].clone());
                i += 1;
            }
            "-p" | "--port" => {
                if i + 1 >= args.len() || looks_like_flag(&args[i + 1]) {
                    fail(c, &format!("missing value for {arg}"));
                    return None;
                }
                match args[i + 1].parse::<u16>() {
                    Ok(p) => opts.port = p,
                    Err(_) => {
                        fail(c, &format!("invalid port number \"{}\"", args[i + 1]));
                        return None;
                    }
                }
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

fn validate_extension(c: Colors, path: &str) {
    let p = Path::new(path);
    let is_du = p
        .extension()
        .map(|ext| ext.eq_ignore_ascii_case("du"))
        .unwrap_or(false);
    if !is_du {
        warn(
            c,
            &format!("input file {path:?} does not have a .du extension"),
        );
    }
}

fn looks_like_flag(s: &str) -> bool {
    s.starts_with('-') && s != "-"
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
