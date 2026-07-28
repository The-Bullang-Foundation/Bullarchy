use bullang::ast::{Backend, Param};

pub const META: (&str, &str, &str) = (
    "replace_str",
    "(s, from, to: String)     → String",
    "Replace occurrences",
);

pub fn emit(params: &[Param], backend: &Backend) -> Result<String, String> {
    let p = super::need("replace_str", params, 3)?;
    Ok(match backend {
        Backend::Rust    => format!(
            "{0}.replace({1}.as_str(), {2}.as_str())",
            p[0], p[1], p[2]
        ),
        Backend::Python  => format!(
            "{}.replace({}, {})",
            super::py_esc(p[0]),
            super::py_esc(p[1]),
            super::py_esc(p[2])
        ),
        Backend::C       => format!(
            "replace_str({}, {}, {})",
            p[0], p[1], p[2]
        ),
        Backend::Cpp     => format!(
            "replace_str({0}, {1}, {2})",
            p[0], p[1], p[2]
        ),
        Backend::Go      => format!(
            "strings.ReplaceAll({}, {}, {})",
            p[0], p[1], p[2]
        ),
        Backend::Java    => format!("{0}.replace({1}, {2})", p[0], p[1], p[2]),
        Backend::Unknown(kw) => return Err(format!(
            "'builtin::replace_str' is not available for unknown backend '{}'", kw
        )),
    })
}
