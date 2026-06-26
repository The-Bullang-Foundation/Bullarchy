mod build;
mod cmd;
mod codegen;
mod init;
mod lsp;
mod readme;
mod stdlib;
mod typecheck;
mod utils;
mod validator;
mod desktop;
mod routes;

use axum::{Router, routing::{get, post}};
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::cors::{CorsLayer, Any};

const PORT: u16 = 7474;

const INDEX_HTML: &str = include_str!("../frontend/index.html");
const APP_JS: &str     = include_str!("../frontend/app.js");
const STYLE_CSS: &str  = include_str!("../frontend/style.css");

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    // LSP mode — always takes priority
    if args.get(1).map(|s| s.as_str()) == Some("lsp") {
        lsp::run();
        return;
    }

    // CLI mode: any argument other than --gui triggers non-interactive CLI
    let force_gui = args.get(1).map(|s| s.as_str()) == Some("--gui");
    let force_cli = args.get(1).map(|s| s.as_str()) == Some("--cli");
    let has_cmd   = args.len() > 1 && !force_gui && !force_cli;

    if has_cmd {
        // Direct command: bullarchy <cmd> [args...] — run and exit
        run_cli_command(&args[1], &args[2..]);
        return;
    }

    if force_cli {
        // Interactive REPL, no GUI
        run_cli_repl();
        return;
    }

    // Default or --gui: launch the graphical interface
    run_gui().await;
}

// ── GUI mode ──────────────────────────────────────────────────────────────────

async fn run_gui() {
    desktop::install_if_first_launch();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/",                   get(serve_index))
        .route("/app.js",             get(serve_js))
        .route("/style.css",          get(serve_css))
        .route("/api/init",           post(routes::handle_init))
        .route("/api/convert",        post(routes::handle_convert))
        .route("/api/fmt",            post(routes::handle_fmt))
        .route("/api/check",          post(routes::handle_check))
        .route("/api/editor-setup",   post(routes::handle_editor_setup))
        .route("/api/update",         post(routes::handle_update))
        .route("/api/add",            post(routes::handle_add))
        .route("/api/blueprint/save", post(routes::handle_blueprint_save))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], PORT));
    let url  = format!("http://localhost:{}", PORT);

    println!("Bullarchy — starting on {}", url);
    println!("Run with --cli for the terminal interface.");

    let url_clone = url.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        let _ = open::that(url_clone);
    });

    let listener = tokio::net::TcpListener::bind(addr).await
        .expect("failed to bind port 7474");
    axum::serve(listener, app).await
        .expect("server error");
}

// ── CLI direct command ────────────────────────────────────────────────────────

fn run_cli_command(cmd: &str, rest: &[String]) {
    let parts: Vec<&str> = std::iter::once(cmd)
        .chain(rest.iter().map(|s| s.as_str()))
        .collect();

    match cmd {
        "help" => print_help(),

        "check" => cmd::cmd_check(),

        "editor-setup" => cmd::cmd_editor_setup(),

        "update" => cmd::cmd_update(),

        "fmt" => {
            let (folder, dry_run) = parse_fmt_args(&parts[1..]);
            cmd::cmd_fmt(folder, dry_run);
        }

        "init" => {
            if parts.len() < 2 {
                eprintln!("  Usage: bullarchy init <name> [--depth N] [--lang ext] [--lib header] [--blueprint file] [--path dir]");
            } else {
                match parse_init_args(&parts[1..]) {
                    Ok((name, depth, blueprint, lang, libs, path)) =>
                        cmd::cmd_init(name, depth, blueprint, lang, libs, path),
                    Err(e) => eprintln!("  {}", e),
                }
            }
        }

        "convert" => {
            let target = parts.get(1).map(|s| PathBuf::from(s));
            let second = parts.get(2).map(|s| s.to_string());
            cmd::cmd_convert(target, second);
        }

        "add" => {
            let args: Vec<&str> = parts[1..].to_vec();
            cmd::cmd_add(&args);
        }

        "remove" => {
            let args: Vec<&str> = parts[1..].to_vec();
            cmd::cmd_remove(&args);
        }

        "--gui" => {
            eprintln!("  Use --gui without other arguments to launch the GUI.");
        }

        other => eprintln!(
            "  Unknown command: '{}'. Run 'bullarchy help' for available commands.",
            other
        ),
    }
}

// ── CLI REPL ──────────────────────────────────────────────────────────────────

fn run_cli_repl() {
    println!("{}", BANNER);

    let update_handle = std::thread::spawn(|| {
        let remote    = cmd::remote_head(cmd::DEFAULT_REPO, "main")?;
        let installed = cmd::installed_hash("bullarchy", cmd::DEFAULT_REPO, "main")?;
        if installed == remote { None }
        else { Some("\nA new version of Bullarchy is available. Run `bullarchy update` to install.".to_string()) }
    });

    if let Ok(Some(msg)) = update_handle.join() {
        println!("{}", msg);
    }

    let mut rl = rustyline::DefaultEditor::new()
        .expect("failed to initialise line editor");

    loop {
        let line = match rl.readline("command -> ") {
            Ok(l)                                             => l,
            Err(rustyline::error::ReadlineError::Eof)        => { println!("Goodbye."); break; }
            Err(rustyline::error::ReadlineError::Interrupted) => continue,
            Err(e) => { eprintln!("Read error: {}", e); break; }
        };
        let line = line.trim();
        if line.is_empty() { continue; }
        let _ = rl.add_history_entry(line);

        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts[0] {
            "exit" => { println!("Goodbye."); break; }
            cmd    => run_cli_command(cmd, &parts[1..].iter().map(|s| s.to_string()).collect::<Vec<_>>()),
        }
    }
}

// ── Help ──────────────────────────────────────────────────────────────────────

fn print_help() {
    println!();
    println!("  Usage:  bullarchy [command] [options]");
    println!();
    println!("  No arguments          Launch the graphical interface");
    println!("  --gui                 Launch the graphical interface explicitly");
    println!("  --cli                 Launch the interactive terminal REPL");
    println!();
    println!("  Available commands:");
    println!();
    println!("    init <n> [options]       Scaffold a new Bullang project");
    println!("      --depth N                   Hierarchy depth 1–6 (default: 2)");
    println!("      --lang ext                  Target language: rs, py, c, cpp, go, java");
    println!("      --lib header                External library (repeatable)");
    println!("      --blueprint file            Init from a blueprint.bu file");
    println!("      --path dir                  Where to create the project");
    println!();
    println!("    convert <path> [lang|output]  Transpile a project or .bu file");
    println!("      -n name                     Output folder name (project mode)");
    println!("      -e ext                      Target language (overrides #lang)");
    println!("      --out dir                   Explicit output path (project mode)");
    println!("      -o file                     Output file (single-file mode)");
    println!();
    println!("    add                         List all available packages");
    println!("    add <name>                  Install a package from the registry");
    println!("    add <name>@<version>        Install a specific version");
    println!("    add <https://...>           Install from a git URL");
    println!("    remove <name>               Uninstall a package");
    println!();
    println!("    fmt [folder] [--dry-run]    Format all .bu files to canonical style");
    println!("    check                       Validate, type-check, and check formatting");
    println!("    editor-setup                Write LSP config for Neovim, Helix, Emacs");
    println!("    update                      Reinstall Bullarchy from the repository");
    println!("    help                        Show this message");
    println!();
    println!("  Note: the LSP server is started directly by editors via `bullarchy lsp`.");
    println!();
}

// ── Argument parsers ──────────────────────────────────────────────────────────

fn parse_fmt_args(args: &[&str]) -> (Option<PathBuf>, bool) {
    let mut folder  = None;
    let mut dry_run = false;
    for arg in args {
        match *arg {
            "--dry-run" => dry_run = true,
            s           => folder = Some(PathBuf::from(s)),
        }
    }
    (folder, dry_run)
}

fn parse_init_args(
    args: &[&str],
) -> Result<(String, u8, Option<PathBuf>, Option<String>, Vec<String>, Option<PathBuf>), String> {
    let name          = args.first().ok_or("missing project name")?.to_string();
    let mut depth     = 2u8;
    let mut lang      = None;
    let mut libs      = Vec::new();
    let mut blueprint = None;
    let mut path      = None;

    let mut i = 1;
    while i < args.len() {
        match args[i] {
            "--depth" => {
                i += 1;
                depth = args.get(i)
                    .ok_or("--depth requires a value")?
                    .parse::<u8>()
                    .map_err(|_| "--depth must be a number between 1 and 6")?;
            }
            "--lang" => {
                i += 1;
                lang = Some(args.get(i).ok_or("--lang requires a value")?.to_string());
            }
            "--lib" => {
                i += 1;
                libs.push(args.get(i).ok_or("--lib requires a value")?.to_string());
            }
            "--blueprint" => {
                i += 1;
                blueprint = Some(PathBuf::from(args.get(i).ok_or("--blueprint requires a value")?));
            }
            "--path" => {
                i += 1;
                path = Some(PathBuf::from(args.get(i).ok_or("--path requires a value")?));
            }
            other => return Err(format!("unknown option: '{}'", other)),
        }
        i += 1;
    }

    Ok((name, depth, blueprint, lang, libs, path))
}

// ── Static file handlers ──────────────────────────────────────────────────────

async fn serve_index() -> axum::response::Html<&'static str> {
    axum::response::Html(INDEX_HTML)
}

async fn serve_js() -> impl axum::response::IntoResponse {
    ([(axum::http::header::CONTENT_TYPE, "application/javascript")], APP_JS)
}

async fn serve_css() -> impl axum::response::IntoResponse {
    ([(axum::http::header::CONTENT_TYPE, "text/css")], STYLE_CSS)
}

// ── Banner (REPL only) ────────────────────────────────────────────────────────

const BANNER: &str = r#"
  ____        _ _               _
 |  _ \      | | |             | |
 | |_) |_   _| | | __ _ _ __ ___| |__  _   _
 |  _ <| | | | | |/ _` | '__/ __| '_ \| | | |
 | |_) | |_| | | | (_| | | | (__| | | | |_| |
 |____/ \__,_|_|_|\__,_|_|  \___|_| |_|\__, |
                                        __/ |
                                       |___/

Bullarchy 1.0.0 — Bullang project toolchain
Type 'help' for available commands. Type 'exit' to quit.
"#;
