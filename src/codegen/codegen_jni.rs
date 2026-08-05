//! JNI companion files for the Java backend's arbitrary-fd support.
//!
//! `open`/`close`/`in`/`out` on a real OS file descriptor has no idiomatic
//! public Java API — only System.in/out/err (fds 0/1/2) do. Getting a raw fd
//! at all requires JNI. Unlike per-file codegen, none of these three files
//! depend on project-specific content: the same `bu_native.c`/`BuNative.java`
//! /`Makefile.native` are correct for every project that needs them, so
//! they're static templates (mirrors `foreign_types.h`'s `include_str!`
//! pattern) rather than generated per source file.

use bullang::ast::*;

pub const BU_NATIVE_C: &str        = include_str!("../bu_native.c");
pub const BU_NATIVE_JAVA: &str     = include_str!("../BuNative.java");
pub const MAKEFILE_NATIVE: &str    = include_str!("../Makefile.native");

const FD_BUILTINS: &[&str] = &["open", "close", "in", "out"];

/// True if `file` needs the JNI native lib: any use of `open` (always —
/// there's no fast path for opening an arbitrary file), or any use of
/// `close`/`in`/`out` whose fd argument isn't provably one of the literals
/// 0/1/2 (those three stay on the System.in/out/err fast path in Java's
/// codegen — see close.rs/fd_in.rs/fd_out.rs). A non-literal fd (a variable,
/// a field access, anything computed) can't be proven safe statically, so
/// it's treated as needing the native lib too.
pub fn needs_jni_java(file: &SourceFile) -> bool {
    file.bullets.iter().any(|b| bullet_needs_jni(&b.body))
}

fn bullet_needs_jni(body: &BulletBody) -> bool {
    match body {
        BulletBody::Pipes(pipes) => pipes.iter().any(|pipe| {
            pipe.inputs.iter().any(expr_needs_jni)
                || match &pipe.expr {
                    Expr::Atom(Atom::BuiltinNoArgs(name)) => call_needs_jni(name, &pipe.inputs),
                    other => expr_needs_jni(other),
                }
        }),
        // Whole-body bare form (`-> builtin::open`, no pipes at all): the fd
        // for close/in/out would come from the function's own params, which
        // this AST node carries no reference to here — conservative.
        BulletBody::Builtin(name) => FD_BUILTINS.contains(&name.as_str()),
        BulletBody::Natives(_) => false,
    }
}

fn call_needs_jni(name: &str, args: &[Expr]) -> bool {
    match name {
        "open" => true,
        "close" | "in" | "out" => match args.first() {
            Some(Expr::Atom(Atom::Integer(0 | 1 | 2))) => false,
            Some(_) => true,
            None => false,
        },
        _ => false,
    }
}

fn expr_needs_jni(expr: &Expr) -> bool {
    match expr {
        Expr::Atom(a) => atom_needs_jni(a),
        Expr::BinOp(b) => atom_needs_jni(&b.lhs) || atom_needs_jni(&b.rhs),
        Expr::Tuple(items) => items.iter().any(expr_needs_jni),
    }
}

fn atom_needs_jni(atom: &Atom) -> bool {
    match atom {
        Atom::BuiltinExpr { name, args } => {
            call_needs_jni(name, args) || args.iter().any(expr_needs_jni)
        }
        // No enclosing pipe's inputs available from here — conservative.
        Atom::BuiltinNoArgs(name) => FD_BUILTINS.contains(&name.as_str()),
        Atom::Unary { rhs, .. } => atom_needs_jni(rhs),
        Atom::Index { idx, .. } => expr_needs_jni(idx),
        Atom::Slice { from, to, .. } => expr_needs_jni(from) || expr_needs_jni(to),
        _ => false,
    }
}
