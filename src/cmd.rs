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
use crate::fmt::format_source;
use crate::init::{scaffold_init, scaffold_new, ProjectConfig};
use crate::parser::Parser;
use crate::sema::analyze;

use xarp::style::Styles;
use xarp::{Arg, ArgAction, ArgMatches, Xarp, XarpError};

/// DocUP compiler version reported by `docup version`, `docup -v`, and `docup --version`.
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

/// Constructs the top-level application CLI definition with `xarp`.
pub fn build_cli() -> Xarp {
    Xarp::new("docup")
        .version(VERSION)
        .about("DocUP Compiler — compiles .du documents to standalone HTML5")
        .styles(Styles::styled())
        .arg(
            Arg::new("v")
                .short('v')
                .action(ArgAction::SetTrue)
                .help("Print version information"),
        )
        .subcommand(
            Xarp::new("build")
                .about("Compile a .du document to standalone HTML5")
                .arg(
                    Arg::new("input")
                        .value_name("input.du")
                        .help("Path to the input .du document")
                        .required(true),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("file")
                        .help("Output HTML path (default: input with .html extension)"),
                )
                .arg(
                    Arg::new("style")
                        .short('s')
                        .long("style")
                        .value_name("file")
                        .help("Custom CSS stylesheet to inject into output HTML"),
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .action(ArgAction::SetTrue)
                        .help("Print detailed timing for each build stage")
                        .conflicts_with("quiet"),
                )
                .arg(
                    Arg::new("quiet")
                        .short('q')
                        .long("quiet")
                        .action(ArgAction::SetTrue)
                        .help("Suppress step-by-step progress output")
                        .conflicts_with("verbose"),
                ),
        )
        .subcommand(
            Xarp::new("watch")
                .about("Watch a .du document and rebuild with live reloading")
                .arg(
                    Arg::new("input")
                        .value_name("input.du")
                        .help("Path to the input .du document")
                        .required(true),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("file")
                        .help("Output HTML path (default: input with .html extension)"),
                )
                .arg(
                    Arg::new("style")
                        .short('s')
                        .long("style")
                        .value_name("file")
                        .help("Custom CSS stylesheet to inject into output HTML"),
                )
                .arg(
                    Arg::new("port")
                        .short('p')
                        .long("port")
                        .value_name("port")
                        .default_value("8080")
                        .help("Local development server port for watch mode (default: 8080)"),
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .action(ArgAction::SetTrue)
                        .help("Print detailed timing for each build stage")
                        .conflicts_with("quiet"),
                )
                .arg(
                    Arg::new("quiet")
                        .short('q')
                        .long("quiet")
                        .action(ArgAction::SetTrue)
                        .help("Suppress step-by-step progress output")
                        .conflicts_with("verbose"),
                ),
        )
        .subcommand(
            Xarp::new("fmt")
                .about("Format .du documents to canonical style")
                .arg(
                    Arg::new("path")
                        .value_name("path")
                        .help("File or directory to format (defaults to current directory)")
                        .default_value("."),
                )
                .arg(
                    Arg::new("check")
                        .long("check")
                        .action(ArgAction::SetTrue)
                        .help(
                        "Check formatting without writing changes to disk (exits 1 if unformatted)",
                    ),
                )
                .arg(
                    Arg::new("stdout")
                        .long("stdout")
                        .action(ArgAction::SetTrue)
                        .help("Print formatted document to stdout (single file only)")
                        .conflicts_with("check"),
                ),
        )
        .subcommand(
            Xarp::new("new")
                .about("Create a new DocUP document project in a new directory")
                .arg(
                    Arg::new("path")
                        .value_name("directory")
                        .help("Target directory name")
                        .required(true),
                )
                .arg(
                    Arg::new("title")
                        .long("title")
                        .value_name("title")
                        .help("Project document title"),
                )
                .arg(
                    Arg::new("author")
                        .long("author")
                        .value_name("name")
                        .help("Author name"),
                )
                .arg(
                    Arg::new("theme")
                        .long("theme")
                        .value_name("theme")
                        .help("Theme name (textbook, default, sepia, nord, solarized)"),
                )
                .arg(
                    Arg::new("desc")
                        .long("desc")
                        .value_name("text")
                        .help("Project description"),
                )
                .arg(
                    Arg::new("yes")
                        .short('y')
                        .long("yes")
                        .action(ArgAction::SetTrue)
                        .help("Skip interactive prompts and use default settings"),
                ),
        )
        .subcommand(
            Xarp::new("init")
                .about("Initialize a DocUP project in the current working directory")
                .arg(
                    Arg::new("title")
                        .long("title")
                        .value_name("title")
                        .help("Project document title"),
                )
                .arg(
                    Arg::new("author")
                        .long("author")
                        .value_name("name")
                        .help("Author name"),
                )
                .arg(
                    Arg::new("theme")
                        .long("theme")
                        .value_name("theme")
                        .help("Theme name (textbook, default, sepia, nord, solarized)"),
                )
                .arg(
                    Arg::new("desc")
                        .long("desc")
                        .value_name("text")
                        .help("Project description"),
                )
                .arg(
                    Arg::new("yes")
                        .short('y')
                        .long("yes")
                        .action(ArgAction::SetTrue)
                        .help("Skip interactive prompts and use default settings"),
                ),
        )
        .subcommand(Xarp::new("version").about("Print version information"))
        .subcommand(Xarp::new("help").about("Print help information"))
}

/// Run is the main CLI entry point called by `main`.
pub fn run(args: &[String]) -> i32 {
    let colors = Colors::detect();
    let cli = build_cli();

    // Reconstruct argv ensuring the application name occupies index 0 for xarp
    let mut argv: Vec<String> = if args.first().map_or(false, |a| {
        a == "docup"
            || a.ends_with("/docup")
            || a.ends_with("\\docup.exe")
            || a.ends_with("\\docup")
    }) {
        args.to_vec()
    } else {
        let mut v = Vec::with_capacity(args.len() + 1);
        v.push("docup".to_string());
        v.extend_from_slice(args);
        v
    };

    if argv.len() <= 1 {
        cli.print_help();
        return 1;
    }

    // Support -V as an alias for -v / --verbose in subcommand invocations
    if argv.len() > 2 && (argv[1] == "build" || argv[1] == "watch") {
        for arg in &mut argv[2..] {
            if arg == "-V" {
                *arg = "-v".to_string();
            }
        }
    }

    let matches = match cli.try_get_matches_from(&argv) {
        Ok(m) => m,
        Err(XarpError::Help(msg)) => {
            print!("{msg}");
            return 0;
        }
        Err(XarpError::Version(msg)) => {
            println!("{msg}");
            return 0;
        }
        Err(XarpError::Parse(err)) => {
            eprintln!("{err}");
            return 1;
        }
    };

    match matches.subcommand() {
        Some(("build", sub_matches)) => match options_from_matches(sub_matches) {
            Ok(opts) => run_build(colors, opts),
            Err(err) => {
                fail(colors, &err);
                1
            }
        },
        Some(("watch", sub_matches)) => match options_from_matches(sub_matches) {
            Ok(opts) => run_watch(colors, opts),
            Err(err) => {
                fail(colors, &err);
                1
            }
        },
        Some(("fmt", sub_matches)) => {
            let path = sub_matches
                .get_one::<String>("path")
                .unwrap_or_else(|| ".".to_string());
            let check = sub_matches.get_flag("check");
            let stdout = sub_matches.get_flag("stdout");
            run_fmt(colors, &path, check, stdout)
        }
        Some(("new", sub_matches)) => {
            let path_str = sub_matches.get_one::<String>("path").unwrap_or_default();
            let target_path = Path::new(&path_str);
            let yes = sub_matches.get_flag("yes");
            let cfg = parse_project_config(sub_matches);
            match scaffold_new(target_path, cfg, yes, colors) {
                Ok(_) => 0,
                Err(err) => {
                    fail(colors, &err);
                    1
                }
            }
        }
        Some(("init", sub_matches)) => {
            let target_path = Path::new(".");
            let yes = sub_matches.get_flag("yes");
            let cfg = parse_project_config(sub_matches);
            match scaffold_init(target_path, cfg, yes, colors) {
                Ok(_) => 0,
                Err(err) => {
                    fail(colors, &err);
                    1
                }
            }
        }
        Some(("version", _)) => {
            println!("docup version {VERSION}");
            0
        }
        Some(("help", _)) => {
            build_cli().print_help();
            0
        }
        _ => {
            if matches.get_flag("v") {
                println!("docup version {VERSION}");
                0
            } else {
                build_cli().print_help();
                1
            }
        }
    }
}

pub fn print_usage(_c: Colors) {
    build_cli().print_help();
}

fn parse_project_config(matches: &ArgMatches) -> Option<ProjectConfig> {
    let title = matches.get_one::<String>("title").unwrap_or_default();
    let author = matches.get_one::<String>("author").unwrap_or_default();
    let theme = matches.get_one::<String>("theme").unwrap_or_default();
    let desc = matches.get_one::<String>("desc").unwrap_or_default();

    if title.is_empty() && author.is_empty() && theme.is_empty() && desc.is_empty() {
        None
    } else {
        let default_cfg = ProjectConfig::default();
        Some(ProjectConfig {
            title: if title.is_empty() {
                default_cfg.title
            } else {
                title
            },
            author: if author.is_empty() {
                default_cfg.author
            } else {
                author
            },
            theme: if theme.is_empty() {
                default_cfg.theme
            } else {
                theme
            },
            description: desc,
            sample_content: true,
        })
    }
}

fn options_from_matches(matches: &ArgMatches) -> Result<Options, String> {
    let input_path = matches.get_one::<String>("input").unwrap_or_default();
    let output_path = matches.get_one::<String>("output").unwrap_or_default();
    let style_path = matches.get_one::<String>("style");
    let port = match matches.try_get_one::<u16>("port") {
        Ok(Some(p)) => p,
        Ok(None) => 8080,
        Err(err) => return Err(err.to_string()),
    };
    let verbose = matches.get_flag("verbose");
    let quiet = matches.get_flag("quiet");

    Ok(Options {
        input_path,
        output_path,
        style_path,
        port,
        verbose,
        quiet,
    })
}

fn run_build(c: Colors, opts: Options) -> i32 {
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

fn run_watch(c: Colors, opts: Options) -> i32 {
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

fn run_fmt(c: Colors, target: &str, check: bool, stdout: bool) -> i32 {
    let target_path = Path::new(target);
    if !target_path.exists() {
        fail(c, &format!("target path does not exist: {target}"));
        return 1;
    }

    let files = if target_path.is_file() {
        if stdout
            && target_path
                .extension()
                .map_or(true, |ext| !ext.eq_ignore_ascii_case("du"))
        {
            warn(c, &format!("file {target:?} does not have a .du extension"));
        }
        vec![target_path.to_path_buf()]
    } else {
        if stdout {
            fail(c, "cannot use --stdout when formatting a directory");
            return 1;
        }
        let mut list = Vec::new();
        if let Err(err) = collect_du_files(target_path, &mut list) {
            fail(c, &format!("failed to traverse directory {target}: {err}"));
            return 1;
        }
        list.sort();
        if list.is_empty() {
            println!("{}No .du files found in {target}.{}", c.dim, c.reset);
            return 0;
        }
        list
    };

    let mut changed_count = 0;
    let mut error_count = 0;

    for path in &files {
        let content = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(err) => {
                report_file_error(c, &path.to_string_lossy(), &err);
                error_count += 1;
                continue;
            }
        };

        let formatted = match format_source(&content) {
            Ok(f) => f,
            Err(err) => {
                report_compile_error(c, &path.to_string_lossy(), content.as_bytes(), &err);
                error_count += 1;
                continue;
            }
        };

        if stdout {
            print!("{formatted}");
            return if error_count > 0 { 1 } else { 0 };
        }

        if formatted != content {
            changed_count += 1;
            if check {
                println!(
                    "  {}needs formatting:{} {}",
                    c.yellow,
                    c.reset,
                    path.display()
                );
            } else {
                if let Err(err) = std::fs::write(path, formatted.as_bytes()) {
                    report_file_error(c, &path.to_string_lossy(), &err);
                    error_count += 1;
                    continue;
                }
                println!("  {}formatted{} {}", c.green, c.reset, path.display());
            }
        }
    }

    if check {
        if changed_count > 0 {
            eprintln!(
                "\n{}✗ {changed_count} file(s) require formatting. Run `docup fmt` to update.{}",
                c.red, c.reset
            );
            return 1;
        }
        if error_count == 0 {
            println!(
                "\n{}{}✓ All files are properly formatted.{}",
                c.bold, c.green, c.reset
            );
        }
    } else if error_count == 0 {
        if changed_count > 0 {
            println!(
                "\n{}{}✓ Successfully formatted {changed_count} file(s).{}",
                c.bold, c.green, c.reset
            );
        } else {
            println!(
                "{}All files already follow canonical style. Nothing to change.{}",
                c.dim, c.reset
            );
        }
    }

    if error_count > 0 {
        1
    } else {
        0
    }
}

fn collect_du_files(dir: &Path, list: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

        // Skip hidden directories and standard build/dependency folders
        if file_name.starts_with('.')
            || file_name == "target"
            || file_name == "node_modules"
            || file_name == "dist"
        {
            continue;
        }

        if path.is_dir() {
            collect_du_files(&path, list)?;
        } else if path
            .extension()
            .map_or(false, |ext| ext.eq_ignore_ascii_case("du"))
        {
            list.push(path);
        }
    }
    Ok(())
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
