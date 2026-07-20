//! `convert` — transpile a project folder or a single .bu file.
//!
//! Usage:
//!   convert my_project          — auto-detect lang from #lang: directive
//!   convert my_project py       — override lang for the whole project
//!   convert file.bu             — auto-detect lang from nearest inventory
//!   convert file.bu rs          — override lang, write next to source
//!   convert file.bu out.rs      — convert and write to explicit output path
//!
//! The second positional argument is interpreted as:
//!   - a known short extension (rs py c cpp go)  → language override
//!   - a filename ending in a known extension     → output path (single-file only)
//!   - absent                                     → auto-detect

use std::path::{Path, PathBuf};
use bullang::ast::{self, Backend};
use crate::validator::{self, AllErrors};
use crate::{build, codegen, typecheck};
use bullang::parser;
use crate::utils::{current_dir, read_file, find_root_from, find_root_from_probe,
                   print_all_errors, print_type_errors};
use crate::readme::delete_project_readme;

// ── Short extension set ───────────────────────────────────────────────────────

const KNOWN_EXTS: &[&str] = &["rs", "py", "c", "cpp", "go"];

fn is_known_ext(s: &str) -> bool { KNOWN_EXTS.contains(&s) }

// ── Public entry point ────────────────────────────────────────────────────────

pub fn cmd_convert(target: Option<PathBuf>, second: Option<String>) {
    let path = match target {
        Some(p) => p,
        None    => current_dir(),
    };

    if path.extension().map(|e| e == "bu").unwrap_or(false) {
        // ── Single-file mode ──────────────────────────────────────────────────
        if !path.exists() {
            eprintln!("error: '{}' not found", path.display());
            std::process::exit(1);
        }
        let input = path.canonicalize().unwrap_or(path);

        let (lang, out_path) = resolve_single_second(&input, second.as_deref());
        cmd_convert_file(input, lang, out_path);
    } else {
        // ── Project mode ──────────────────────────────────────────────────────
        let source_dir = if path.exists() && path.is_dir() {
            path.canonicalize().unwrap_or(path)
        } else {
            eprintln!("error: '{}' is not a directory or .bu file", path.display());
            std::process::exit(1);
        };

        let lang_override = second.map(|s| {
            if is_known_ext(&s) { s }
            else {
                eprintln!("error: '{}' is not a recognised backend — use rs py c cpp go java", s);
                std::process::exit(1);
            }
        });

        cmd_convert_project(source_dir, lang_override);
    }
}

/// Resolve the second positional arg for single-file mode.
/// Returns (lang_ext, Option<explicit_output_path>).
fn resolve_single_second(input: &Path, second: Option<&str>) -> (String, Option<PathBuf>) {
    match second {
        None => {
            // Auto-detect from nearest inventory #lang
            let lang = detect_lang_for_file(input);
            (lang, None)
        }
        Some(s) if is_known_ext(s) => {
            // Pure language override, output next to source
            (s.to_string(), None)
        }
        Some(s) => {
            // Must be an output path — infer lang from its extension
            let out = PathBuf::from(s);
            let lang = out.extension()
                .and_then(|e| e.to_str())
                .filter(|e| is_known_ext(e))
                .unwrap_or_else(|| {
                    eprintln!("error: cannot infer language from '{}' — \
                               use a known extension (rs, py, c, cpp, go)", s);
                    std::process::exit(1);
                })
                .to_string();
            (lang, Some(out))
        }
    }
}

fn detect_lang_for_file(input: &Path) -> String {
    if let Some(dir) = input.parent() {
        if let Ok(inv) = validator::read_inventory(dir) {
            if let Some(ref b) = inv.lang { return b.ext().to_string(); }
        }
        let probe = find_root_from_probe(dir);
        if let Ok(inv) = validator::read_inventory(&probe) {
            if let Some(ref b) = inv.lang { return b.ext().to_string(); }
        }
    }
    "rs".to_string()
}

// ── Single-file conversion ────────────────────────────────────────────────────

fn cmd_convert_file(input: PathBuf, lang: String, explicit_out: Option<PathBuf>) {
    let source = read_file(&input);
    let is_inv = input.file_name().and_then(|n| n.to_str())
        .map(|n| n == "inventory.bu").unwrap_or(false);

    let bu = parser::parse_file(&source, is_inv).unwrap_or_else(|e| {
        eprintln!("parse error in {}:\n  {}", input.display(), e);
        std::process::exit(1);
    });

    let backend = Backend::from_ext(&lang).unwrap_or(Backend::Rust);
    let stem    = input.file_stem().and_then(|s| s.to_str()).unwrap_or("out");
    let out_dir = input.parent().unwrap_or_else(|| Path::new("."));

    let out_path = |ext: &str| -> PathBuf {
        explicit_out.clone()
            .unwrap_or_else(|| out_dir.join(format!("{}.{}", stem, ext)))
    };

    match bu {
        ast::BuFile::Source(ref sf) => {
            // Check for escape block conflicts
            check_escape_compat(sf, &backend, &input);

            let (content, ext) = match backend {
                Backend::Rust        => (codegen::emit_bare_rs(sf),   "rs"),
                Backend::Python      => (codegen::emit_bare_py(sf),   "py"),
                Backend::C           => (codegen::emit_bare_c(sf),    "c"),
                Backend::Cpp         => (codegen::emit_bare_cpp(sf),  "cpp"),
                Backend::Go          => (codegen::emit_bare_go(sf),   "go"),
                Backend::Java        => (codegen::emit_bare_java(sf), "java"),
                Backend::Unknown(_)  => (codegen::emit_bare_rs(sf),   "rs"),
            };
            let out = out_path(ext);
            write_or_exit(&out, content);
            println!("wrote {}", out.display());
        }
        ast::BuFile::Inventory(_) => {
            let out = out_path("rs");
            write_or_exit(&out, codegen::emit_mod_rs(&[]));
            println!("wrote {}", out.display());
        }
    }
}

fn check_escape_compat(sf: &ast::SourceFile, backend: &Backend, path: &Path) {
    for bullet in &sf.bullets {
        if let ast::BulletBody::Natives(blocks) = &bullet.body {
            if let Some(block) = blocks.first() {
                if block.backend != *backend
                    && !matches!(block.backend, ast::Backend::Unknown(_))
                {
                    eprintln!(
                        "error: '{}': function '{}' has a @{} escape block \
                         but target is {}. Remove the override or match the backend.",
                        path.display(), bullet.name,
                        block.backend.escape_keyword(), backend.escape_keyword()
                    );
                    std::process::exit(1);
                }
            }
        }
    }
}

// ── Project conversion ────────────────────────────────────────────────────────

fn cmd_convert_project(source_dir: PathBuf, lang_override: Option<String>) {
    let root = find_root_from(&source_dir);

    let langs = collect_folder_langs(&root);
    let unique_langs: std::collections::HashSet<String> = langs.values()
        .filter_map(|l| l.as_ref().map(|b| b.ext().to_string()))
        .collect();

    let is_multi = unique_langs.len() > 1;

    if is_multi && lang_override.is_some() {
        eprintln!("error: this project uses multiple languages ({}).",
            unique_langs.into_iter().collect::<Vec<_>>().join(", "));
        eprintln!("       Omit the language argument to convert each folder independently.");
        std::process::exit(1);
    }

    if is_multi {
        cmd_convert_multi(&root, &source_dir);
        return;
    }

    // Single-language project
    let resolved_lang = lang_override.unwrap_or_else(|| {
        let probe = find_root_from_probe(&source_dir);
        validator::read_inventory(&probe)
            .ok()
            .and_then(|inv| inv.lang.map(|b| b.ext().to_string()))
            .unwrap_or_else(|| "rs".to_string())
    });

    let backend = Backend::from_ext(&resolved_lang).unwrap_or_else(|| {
        eprintln!("error: unknown backend '{}' — use rs py c cpp go java", resolved_lang);
        std::process::exit(1);
    });

    let source_name = source_dir.file_name()
        .and_then(|n| n.to_str()).unwrap_or("bullang_project");
    let out_dir = source_dir.parent()
        .unwrap_or(&source_dir)
        .join(format!("_{}", source_name));

    if out_dir.starts_with(&root) || root.starts_with(&out_dir) {
        eprintln!("error: output must be outside the source tree");
        std::process::exit(1);
    }

    let crate_name = out_dir.file_name()
        .and_then(|n| n.to_str()).unwrap_or("bullang_out").to_string();
    let root_rank = validator::read_folder_rank(&root).expect("root has no rank");

    println!("convert");
    println!("  source  : {} ({})", root.display(), root_rank.name());
    println!("  output  : {}", out_dir.display());
    println!("  backend : {}", backend.escape_keyword());
    println!();

    let all_errors = validator::validate_tree(&root);
    if !all_errors.is_empty() { print_all_errors(&all_errors); std::process::exit(1); }
    println!("structural validation ... ok");

    let compat_errors = build::validate_backend_compatibility(&root, &backend);
    if !compat_errors.is_empty() {
        let all = AllErrors { parse: vec![], structural: compat_errors };
        print_all_errors(&all);
        std::process::exit(1);
    }

    let type_errors = typecheck::typecheck_tree(&root);
    if !type_errors.is_empty() { print_type_errors(&type_errors); std::process::exit(1); }
    println!("type checking         ... ok");

    let result = build::build(&root, &out_dir, &crate_name, &backend);
    if !result.errors.is_empty() {
        let all = AllErrors { parse: vec![], structural: result.errors };
        print_all_errors(&all);
        eprintln!("\nconvert failed");
        std::process::exit(1);
    }

    println!("code generation       ... ok\n");
    delete_project_readme(&root);
    println!("wrote {} file(s) to {}", result.files_written, out_dir.display());
    println!();
    print_next_steps(&backend, &out_dir, &crate_name);
}

fn print_next_steps(backend: &Backend, out_dir: &Path, crate_name: &str) {
    match backend {
        Backend::Rust   => println!("to compile:\n  cd {} && cargo build", out_dir.display()),
        Backend::Python => println!("to run:\n  cd {} && python3 -m {}", out_dir.display(), crate_name),
        Backend::C      => println!("to compile:\n  cd {} && make", out_dir.display()),
        Backend::Cpp    => println!("to compile:\n  cd {} && make", out_dir.display()),
        Backend::Go     => println!("to run:\n  cd {} && go run .", out_dir.display()),
        Backend::Java   => println!("to compile:\n  cd {} && javac *.java && java Main", out_dir.display()),
        Backend::Unknown(kw) => eprintln!("error: unknown backend '{}'", kw),
    }
}

// ── Multi-language project ────────────────────────────────────────────────────

fn collect_folder_langs(root: &Path) -> std::collections::HashMap<PathBuf, Option<Backend>> {
    let mut map = std::collections::HashMap::new();
    collect_langs_recursive(root, None, &mut map);
    map
}

fn collect_langs_recursive(
    dir: &Path, parent_lang: Option<&Backend>,
    map: &mut std::collections::HashMap<PathBuf, Option<Backend>>,
) {
    let inv      = validator::read_inventory(dir).ok();
    let own_lang = inv.as_ref().and_then(|i| i.lang.as_ref());
    let effective = own_lang.or(parent_lang);
    map.insert(dir.to_path_buf(), effective.cloned());
    for subdir in validator::collect_subdirs(dir) {
        collect_langs_recursive(&subdir, effective, map);
    }
}

fn cmd_convert_multi(root: &Path, source_dir: &Path) {
    println!("convert (multi-language)");
    println!("  source : {}\n", root.display());

    let mut total = 0usize;
    let mut converted = Vec::new();

    for subdir in validator::collect_subdirs(root) {
        let inv = match validator::read_inventory(&subdir) { Ok(i) => i, Err(_) => continue };
        let backend = match &inv.lang { Some(b) => b.clone(), None => continue };

        let folder_name = subdir.file_name().and_then(|n| n.to_str()).unwrap_or("out");
        let out_dir    = source_dir.join(format!("_{}", folder_name));
        let crate_name = format!("_{}", folder_name);

        println!("  [{} → {}]", backend.escape_keyword(), out_dir.display());

        let all_errors = validator::validate_tree(&subdir);
        if !all_errors.is_empty() { print_all_errors(&all_errors); eprintln!("  skipped {}\n", folder_name); continue; }

        let type_errors = typecheck::typecheck_tree(&subdir);
        if !type_errors.is_empty() { print_type_errors(&type_errors); eprintln!("  skipped {}\n", folder_name); continue; }

        let result = build::build(&subdir, &out_dir, &crate_name, &backend);
        if !result.errors.is_empty() {
            let all = AllErrors { parse: vec![], structural: result.errors };
            print_all_errors(&all);
            eprintln!("  skipped {}\n", folder_name);
            continue;
        }

        total += result.files_written;
        converted.push((folder_name.to_string(), backend.escape_keyword().to_string(), out_dir));
        println!("  wrote {} file(s)\n", result.files_written);
    }

    delete_project_readme(root);

    println!("done — {} converted:", converted.len());
    for (name, lang, out) in &converted {
        println!("  [{}] {} → {}", lang, name, out.display());
    }
    println!("\ntotal files written: {}", total);
}

fn write_or_exit(path: &Path, content: String) {
    let formatted = format_source(path, &content)
        .unwrap_or(content);
    std::fs::write(path, &formatted).unwrap_or_else(|e| {
        eprintln!("error writing {}: {}", path.display(), e);
        std::process::exit(1);
    });
}

/// Run the appropriate code formatter on generated source files.
/// Falls back silently to unformatted content if the formatter is not installed.
fn format_source(path: &Path, content: &str) -> Option<String> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "rs"              => run_formatter(&["rustfmt", "--edition", "2024"], content),
        "py"              => run_formatter(&["black", "-q", "-"], content),
        "go"              => run_formatter(&["gofmt"], content),
        "c" | "cpp" | "h" | "hpp" => run_formatter(&["clang-format", "--style=LLVM"], content),
        _                 => None,
    }
}

fn run_formatter(cmd: &[&str], content: &str) -> Option<String> {
    use std::io::Write;
    let mut child = std::process::Command::new(cmd[0])
        .args(&cmd[1..])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    child.stdin.take()?.write_all(content.as_bytes()).ok()?;
    let output = child.wait_with_output().ok()?;
    if output.status.success() && !output.stdout.is_empty() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}
