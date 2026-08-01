use bullang::ast::{Backend, Param};

pub const META: (&str, &str, &str) = (
    "min",
    "(arr: Vec[T])             → T",
    "Minimum value in an array — delegates the scan to the target language's native tool where one exists",
);

// Empty array behaviour (consistent across all backends, unchanged from
// before this delegated to native tools):
//   - Rust:   panic with a clear message
//   - Python: raises ValueError
//   - C:      returns INT64_MIN (sentinel — caller must check)
//   - C++:    returns INT64_MIN
//   - Go:     returns math.MinInt64
//   - Java:   returns Long.MIN_VALUE

pub fn emit(params: &[Param], backend: &Backend) -> Result<String, String> {
    let p = super::need("min", params, 1)?;
    let arr = p[0];

    Ok(match backend {
        // ── Rust ─────────────────────────────────────────────────────────────
        Backend::Rust => format!(
            "{arr}.iter().min().cloned().unwrap_or_else(|| panic!(\"builtin::min called on empty Vec\"))"
        ),

        // ── Python ───────────────────────────────────────────────────────────
        Backend::Python => format!("min({})", super::py_esc(arr)),

        // ── C ────────────────────────────────────────────────────────────────
        // No native "min of an array" tool exists in C — hand-rolled scan
        // is the correct approach here, not a reinvented wheel.
        Backend::C => format!(
            "({{ \
               vec_t *__src = {arr}; \
               size_t __n = vec_len(__src); \
               int64_t __m = INT64_MIN; \
               if (__n > 0) {{ \
                 __m = *(int64_t *)vec_get(__src, 0); \
                 for (size_t __i = 1; __i < __n; __i++) {{ \
                   int64_t __v = *(int64_t *)vec_get(__src, __i); \
                   if (__v < __m) __m = __v; \
                 }} \
               }} \
               __m; \
             }})"
        ),

        // ── C++ ──────────────────────────────────────────────────────────────
        Backend::Cpp => format!(
            "[&]() -> int64_t {{ \
               const auto &__v = {arr}; \
               if (__v.empty()) return INT64_MIN; \
               return static_cast<int64_t>(*std::min_element(__v.begin(), __v.end())); \
             }}()"
        ),

        // ── Go ───────────────────────────────────────────────────────────────
        Backend::Go => format!(
            "func() int64 {{ \
               __v := {arr}; \
               if len(__v) == 0 {{ return math.MinInt64 }} \
               return slices.Min(__v); \
             }}()"
        ),

        // ── Java ─────────────────────────────────────────────────────────────
        Backend::Java => format!(
            "({arr}.isEmpty() ? Long.MIN_VALUE : java.util.Collections.min({arr}))"
        ),
        Backend::Unknown(kw) => return Err(format!(
            "'builtin::min' is not available for unknown backend '{kw}'"
        )),
    })
}
