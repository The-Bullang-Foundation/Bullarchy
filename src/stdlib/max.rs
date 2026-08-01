use bullang::ast::{Backend, Param};

pub const META: (&str, &str, &str) = (
    "max",
    "(arr: Vec[T])             → T",
    "Maximum value in an array — delegates the scan to the target language's native tool where one exists",
);

// Empty array behaviour (consistent across all backends, unchanged from
// before this delegated to native tools):
//   - Rust:   panic with a clear message
//   - Python: raises ValueError
//   - C:      returns INT64_MAX (sentinel — caller must check)
//   - C++:    returns INT64_MAX
//   - Go:     returns math.MaxInt64
//   - Java:   returns Long.MAX_VALUE

pub fn emit(params: &[Param], backend: &Backend) -> Result<String, String> {
    let p = super::need("max", params, 1)?;
    let arr = p[0];

    Ok(match backend {
        // ── Rust ─────────────────────────────────────────────────────────────
        // Iterator::max() is the native tool — no hand-rolled scan needed.
        Backend::Rust => format!(
            "{arr}.iter().max().cloned().unwrap_or_else(|| panic!(\"builtin::max called on empty Vec\"))"
        ),

        // ── Python ───────────────────────────────────────────────────────────
        // max() already raises ValueError on an empty sequence — exactly the
        // documented behaviour, with zero extra code.
        Backend::Python => format!("max({})", super::py_esc(arr)),

        // ── C ────────────────────────────────────────────────────────────────
        // No native "max of an array" tool exists in C — hand-rolled scan
        // is the correct approach here, not a reinvented wheel.
        Backend::C => format!(
            "({{ \
               vec_t *__src = {arr}; \
               size_t __n = vec_len(__src); \
               int64_t __m = INT64_MAX; \
               if (__n > 0) {{ \
                 __m = *(int64_t *)vec_get(__src, 0); \
                 for (size_t __i = 1; __i < __n; __i++) {{ \
                   int64_t __v = *(int64_t *)vec_get(__src, __i); \
                   if (__v > __m) __m = __v; \
                 }} \
               }} \
               __m; \
             }})"
        ),

        // ── C++ ──────────────────────────────────────────────────────────────
        // std::max_element is the native tool for the scan itself; the
        // empty-vector guard is still needed since dereferencing end() is UB.
        Backend::Cpp => format!(
            "[&]() -> int64_t {{ \
               const auto &__v = {arr}; \
               if (__v.empty()) return INT64_MAX; \
               return static_cast<int64_t>(*std::max_element(__v.begin(), __v.end())); \
             }}()"
        ),

        // ── Go ───────────────────────────────────────────────────────────────
        // slices.Max is the native tool for the scan; it panics on an empty
        // slice, so the guard preserves the documented sentinel behaviour.
        Backend::Go => format!(
            "func() int64 {{ \
               __v := {arr}; \
               if len(__v) == 0 {{ return math.MaxInt64 }} \
               return slices.Max(__v); \
             }}()"
        ),

        // ── Java ─────────────────────────────────────────────────────────────
        // Collections.max is the native tool; it throws on an empty
        // collection, so the guard preserves the documented sentinel.
        Backend::Java => format!(
            "({arr}.isEmpty() ? Long.MAX_VALUE : java.util.Collections.max({arr}))"
        ),
        Backend::Unknown(kw) => return Err(format!(
            "'builtin::max' is not available for unknown backend '{kw}'"
        )),
    })
}
