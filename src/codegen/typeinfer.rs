//! Minimal, best-effort local type tracking for builtin-call emission.
//!
//! The Bullang AST (`Pipe`) carries no type annotations at all — bindings
//! are just an `Option<String>` name, nothing more (confirmed against the
//! `bullang` crate's `ast.rs` directly). Historically every builtin-call
//! site hardcoded `Param { ty: BuType::Unknown, .. }` when constructing the
//! synthetic params passed into `stdlib::emit_builtin`, so a builtin that
//! genuinely needs to know its own argument types (e.g. `swap` needing to
//! construct a concrete `Tuple[T,T]` value) had no way to do so — which is
//! exactly how `swap`'s C and Go emission ended up broken.
//!
//! This is intentionally NOT a full type-checker: it only resolves the
//! cases needed to unblock type-dependent builtin emission — identifiers
//! already in scope (the enclosing function's own params, or an earlier
//! same-file pipe binding), calls to other same-file functions (via their
//! declared output type), and literals. Anything else — and any case a
//! future maintainer didn't anticipate — resolves to `BuType::Unknown`.
//! Callers that need a concrete type should treat `Unknown` as "couldn't
//! infer" and fail loudly (return an `Err` from `emit()`, surfaced as a
//! visible `/* ERROR */` comment) rather than guess and silently emit the
//! wrong shape — that silent failure is exactly the bug class this exists
//! to close off.

use bullang::ast::*;
use std::collections::HashMap;

/// Declared output type of every function in `file`, keyed by name.
///
/// Same-file only — mirrors the existing same-file limitation already
/// documented on `collect_unit_functions` elsewhere in codegen_c.rs. In a
/// multi-file project build, a caller in `main.bu` invoking a function
/// declared in a different `.bu` module won't have its type resolved by
/// this (falls back to `Unknown`) — that cross-file case isn't covered
/// here either, consistent with the rest of this codebase's current scope.
pub fn collect_fn_output_types(file: &SourceFile) -> HashMap<String, BuType> {
    file.bullets.iter()
        .map(|f| (
            f.name.clone(),
            f.output.as_ref().map(|o| o.ty.clone())
                .unwrap_or_else(|| BuType::Named("()".to_string())),
        ))
        .collect()
}

/// Tracks locally-known variable types while walking a function's `Pipes`
/// body in declaration order, so builtin-call emission can ask "what type
/// is this argument" instead of assuming `Unknown`.
pub struct TypeEnv<'a> {
    vars: HashMap<String, BuType>,
    fn_outputs: &'a HashMap<String, BuType>,
}

impl<'a> TypeEnv<'a> {
    /// Seed from the enclosing function's own declared parameters — the
    /// one place types are always known for certain, since `Param` (unlike
    /// `Pipe`) does carry a real `ty: BuType` field.
    pub fn seed(params: &[Param], fn_outputs: &'a HashMap<String, BuType>) -> Self {
        let vars = params.iter().map(|p| (p.name.clone(), p.ty.clone())).collect();
        TypeEnv { vars, fn_outputs }
    }

    /// Best-effort type of `expr` given what's known so far. `Unknown`
    /// means "couldn't resolve", not "resolved to some universal type" —
    /// treat it as a hard "don't know", never as a stand-in for a real type.
    pub fn infer(&self, expr: &Expr) -> BuType {
        match expr {
            Expr::Atom(Atom::Ident(name)) =>
                self.vars.get(name).cloned().unwrap_or(BuType::Unknown),
            Expr::Atom(Atom::Call { name, .. }) =>
                self.fn_outputs.get(name).cloned().unwrap_or(BuType::Unknown),
            Expr::Atom(Atom::Integer(_)) => BuType::Named("i64".to_string()),
            Expr::Atom(Atom::Float(_))   => BuType::Named("f64".to_string()),
            Expr::Atom(Atom::StringLit(_)) | Expr::Atom(Atom::Interp(_)) =>
                BuType::Named("String".to_string()),
            _ => BuType::Unknown,
        }
    }

    /// Record `name`'s inferred type after a pipe binds it.
    pub fn bind(&mut self, name: &str, ty: BuType) {
        self.vars.insert(name.to_string(), ty);
    }
}
