//! Standard library — universal builtin functions.
//!
//! The 11 math/sorting builtins (abs, pow, powf, sqrt, clamp, log, exp,
//! insertion_sort, quick_sort, merge_sort, radix_sort) have been moved to
//! the external `bull-mathlib` crate, installable via `bullarchy add mathlib`.
//!
//! Syntax in source files:  builtin::abs   builtin::to_upper   etc.
//!
//! Each builtin lives in its own submodule and exposes:
//!   - `META : (&str, &str, &str)`  — (name, signature, description)
//!   - `emit(params, backend)`      — code-generation entry point
//!
//! Backends: Rust, Python, C, C++, Go, Java

use bullang::ast::{Backend, Param};

mod args;
mod close;
mod ends_with;
mod env;
mod exit;
mod fd_in;
mod fd_out;
mod len;
mod max;
mod min;
mod open;
mod parse_i64;
mod replace_str;
mod run;
mod sleep;
mod starts_with;
mod swap;
mod tern;
mod time;
mod to_lower;
mod to_string;
mod to_upper;
mod trim;

// ── Universal builtin set ─────────────────────────────────────────────────────

pub const BUILTINS: &[(&str, &str, &str)] = &[
    // math (min/max stay in core stdlib)
    min::META,
    max::META,
    // conditions
    tern::META,
    // string
    to_upper::META,
    to_lower::META,
    trim::META,
    starts_with::META,
    ends_with::META,
    replace_str::META,
    to_string::META,
    parse_i64::META,
    len::META,
    // algorithms
    swap::META,
    // io
    fd_in::META,
    fd_out::META,
    open::META,
    close::META,
    time::META,
    // system
    args::META,
    run::META,
    exit::META,
    env::META,
    sleep::META,
];

/// Returns true if `name` is a known core builtin OR a mathlib builtin
/// (when the mathlib feature is enabled).
pub fn is_known_builtin(name: &str) -> bool {
    if BUILTINS.iter().any(|(n, _, _)| *n == name) {
        return true;
    }
    #[cfg(feature = "mathlib")]
    if bull_mathlib::is_known_builtin(name) {
        return true;
    }
    #[cfg(feature = "netlib")]
    if bull_netlib::is_known_builtin(name) {
        return true;
    }
    false
}

// ── Hoisted imports ────────────────────────────────────────────────────────────

/// Rust `use` lines a builtin needs at file scope, for backends where the
/// emitted expression itself no longer inlines them (currently just Rust —
/// see codegen::collect_rust_imports, which hoists these to the top of the
/// generated file instead of repeating them at every call site).
pub fn required_imports(name: &str, backend: &Backend) -> Vec<&'static str> {
    if !matches!(backend, Backend::Rust) {
        return Vec::new();
    }
    match name {
        "in"  => vec!["use std::io::{BufRead, BufReader};", "use std::os::unix::io::FromRawFd;"],
        "out" => vec!["use std::io::Write;", "use std::os::unix::io::FromRawFd;", "use std::mem::ManuallyDrop;"],
        "open" => vec!["use std::os::unix::io::IntoRawFd;"],
        _ => Vec::new(),
    }
}

/// C/C++ `#include` lines a builtin needs at file scope, for backends where
/// the emitted expression calls a stdlib function instead of hand-rolling
/// the logic (e.g. `toupper()` needs `<ctype.h>`). Hoisted once to the top
/// of the file by codegen_c::collect_c_includes / codegen_cpp's equivalent,
/// same pattern as required_imports above for Rust.
pub fn required_includes(name: &str, backend: &Backend) -> Vec<&'static str> {
    match (name, backend) {
        ("to_upper", Backend::C) | ("to_lower", Backend::C) => vec!["#include <ctype.h>"],
        ("trim", Backend::C) => vec!["#include <ctype.h>", "#include <string.h>"],
        ("out", Backend::C) => vec!["#include <stdint.h>", "#include <string.h>", "#include <unistd.h>"],
        ("to_upper", Backend::Cpp) | ("to_lower", Backend::Cpp) => vec!["#include <algorithm>", "#include <cctype>", "#include <string>"],
        ("trim", Backend::Cpp) => vec!["#include <string>"],
        ("out", Backend::Cpp) => vec!["#include <cstdint>", "#include <string>", "#include <unistd.h>"],
        _ => Vec::new(),
    }
}

// ── Hoisted helper functions ─────────────────────────────────────────────────

/// Standalone top-level C/C++ function a builtin needs defined once at file
/// scope — `static` linkage so it's safe to drop into every generated file
/// that uses it without risking a duplicate-symbol link error. Hoisted
/// above the user's own functions by codegen_c::collect_c_helpers /
/// codegen_cpp's equivalent, same call site the builtin's `emit()` then
/// reduces to a plain `name(args)` call instead of inlining the logic
/// (a GNU statement-expression in C, an IIFE lambda in C++).
pub fn helper_definition(name: &str, backend: &Backend) -> Option<&'static str> {
    match (name, backend) {
        ("to_upper", Backend::C) => Some(
            "static char *to_upper(char *str) {\n\
             \tfor (int i = 0; str[i]; i++) {\n\
             \t\tstr[i] = (char)toupper((unsigned char)str[i]);\n\
             \t}\n\
             \treturn (str);\n\
             }\n"
        ),
        ("to_lower", Backend::C) => Some(
            "static char *to_lower(char *str) {\n\
             \tfor (int i = 0; str[i]; i++) {\n\
             \t\tstr[i] = (char)tolower((unsigned char)str[i]);\n\
             \t}\n\
             \treturn (str);\n\
             }\n"
        ),
        ("trim", Backend::C) => Some(
            "static char *trim(char *str) {\n\
             \tint start = 0;\n\
             \twhile (str[start] && isspace((unsigned char)str[start])) {\n\
             \t\tstart++;\n\
             \t}\n\
             \tint end = (int)strlen(str);\n\
             \twhile (end > start && isspace((unsigned char)str[end - 1])) {\n\
             \t\tend--;\n\
             \t}\n\
             \tstr[end] = '\\0';\n\
             \treturn (str + start);\n\
             }\n"
        ),
        ("to_upper", Backend::Cpp) => Some(
            "static std::string to_upper(std::string str) {\n\
             \tstd::transform(str.begin(), str.end(), str.begin(), ::toupper);\n\
             \treturn (str);\n\
             }\n"
        ),
        ("to_lower", Backend::Cpp) => Some(
            "static std::string to_lower(std::string str) {\n\
             \tstd::transform(str.begin(), str.end(), str.begin(), ::tolower);\n\
             \treturn (str);\n\
             }\n"
        ),
        ("trim", Backend::Cpp) => Some(
            "static std::string trim(std::string str) {\n\
             \tstr.erase(0, str.find_first_not_of(\" \\t\\n\\r\"));\n\
             \tstr.erase(str.find_last_not_of(\" \\t\\n\\r\") + 1);\n\
             \treturn (str);\n\
             }\n"
        ),
        _ => None,
    }
}

// ── Dispatch ──────────────────────────────────────────────────────────────────

pub fn emit_builtin(name: &str, params: &[Param], backend: &Backend) -> Result<String, String> {
    // Try core stdlib first
    match name {
        "min"            => return min::emit(params, backend),
        "max"            => return max::emit(params, backend),
        "tern"           => return tern::emit(params, backend),
        "to_upper"       => return to_upper::emit(params, backend),
        "to_lower"       => return to_lower::emit(params, backend),
        "trim"           => return trim::emit(params, backend),
        "starts_with"    => return starts_with::emit(params, backend),
        "ends_with"      => return ends_with::emit(params, backend),
        "replace_str"    => return replace_str::emit(params, backend),
        "to_string"      => return to_string::emit(params, backend),
        "parse_i64"      => return parse_i64::emit(params, backend),
        "len"            => return len::emit(params, backend),
        "swap"           => return swap::emit(params, backend),
        "in"             => return fd_in::emit(params, backend),
        "out"            => return fd_out::emit(params, backend),
        "open"           => return open::emit(params, backend),
        "close"          => return close::emit(params, backend),
        "time"           => return time::emit(params, backend),
        "args"           => return args::emit(params, backend),
        "exit"           => return exit::emit(params, backend),
        "env"            => return env::emit(params, backend),
        "sleep"          => return sleep::emit(params, backend),
        "run"            => return run::emit(params, backend),
        _ => {}
    }

    // Try mathlib (only compiled in when --features mathlib is set)
    #[cfg(feature = "mathlib")]
    if bull_mathlib::is_known_builtin(name) {
        return bull_mathlib::emit(name, params, backend);
    }

    // Try netlib (only compiled in when --features netlib is set)
    #[cfg(feature = "netlib")]
    if bull_netlib::is_known_builtin(name) {
        return bull_netlib::emit(name, params, backend);
    }

    Err(format!(
        "'builtin::{}' is not a known builtin. \
         Run `bullang stdlib --list` to see available builtins. \
         If this is a math/sort function, install mathlib: `bullarchy add mathlib`.",
        name
    ))
}

// ── Shared helpers (private to this module; accessible to all submodules) ─────

fn p(params: &[Param]) -> Vec<&str> {
    params.iter().map(|p| p.name.as_str()).collect()
}

/// Escape a param name that might collide with a Python reserved word.
fn py_esc(name: &str) -> &str {
    match name {
        "from"   => "from_",   "import" => "import_", "class"  => "class_",
        "return" => "return_", "pass"   => "pass_",   "for"    => "for_",
        "while"  => "while_",  "in"     => "in_",     "not"    => "not_",
        "and"    => "and_",    "or"     => "or_",     "if"     => "if_",
        "else"   => "else_",   "lambda" => "lambda_", "with"   => "with_",
        "as"     => "as_",     "try"    => "try_",    "except" => "except_",
        "raise"  => "raise_",  "del"    => "del_",
        other    => other,
    }
}

/// Assert `params` has exactly `n` entries; return their name slices.
fn need<'a>(name: &str, params: &'a [Param], n: usize) -> Result<Vec<&'a str>, String> {
    let v = p(params);
    if v.len() != n {
        return Err(format!(
            "'builtin::{}' requires {} parameter(s) but the function declares {}",
            name, n, v.len()
        ));
    }
    Ok(v)
}
