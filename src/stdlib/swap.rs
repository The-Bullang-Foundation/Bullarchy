use bullang::ast::{Backend, Param};

pub const META: (&str, &str, &str) = (
    "swap",
    "(a: T, b: T)              → Tuple[T, T]  (b, a)",
    "Swap two values — returns them in reversed order as a tuple",
);

pub fn emit(params: &[Param], backend: &Backend) -> Result<String, String> {
    let p = super::need("swap", params, 2)?;
    let (a, b) = (p[0], p[1]);
    let p_types: Vec<&bullang::ast::BuType> = params.iter().map(|param| &param.ty).collect();

    Ok(match backend {
        // Block expression: move both into temporaries, return swapped tuple.
        Backend::Rust => format!(
            "{{ let __swap_a = {a}; let __swap_b = {b}; (__swap_b, __swap_a) }}"
        ),

        // Python tuple packing — idiomatic and evaluation-order-safe.
        Backend::Python => {
            let a = super::py_esc(a);
            let b = super::py_esc(b);
            format!("({b}, {a})")
        }

        // GCC compound statement, evaluating to a `(Tuple_T_T){ .v0 = b, .v1 = a }`
        // compound literal — matches how the rest of the C backend represents
        // Tuple[T,U] values (see codegen_c.rs's tuple_c_name/emit_tuple_struct_c).
        // Needs a's concrete type to know which Tuple_T_T struct to build;
        // that's threaded in via TypeEnv (see codegen.rs's typeinfer module) —
        // if it couldn't be resolved, fail loudly instead of emitting the old
        // broken form (which mutated a/b as lvalues and returned a bare scalar,
        // not a tuple at all).
        Backend::C => {
            let ty = p_types[0];
            if matches!(ty, bullang::ast::BuType::Unknown) {
                return Err(format!(
                    "'builtin::swap' (C backend) couldn't determine the type of '{a}' — \
                     this needs a concrete type to build the returned Tuple[T,T] value"
                ));
            }
            let tuple_name = crate::codegen::codegen_c::tuple_c_name(&[ty.clone(), ty.clone()]);
            format!(
                "({{ __typeof__({a}) __sa = ({a}); \
                      __typeof__({b}) __sb = ({b}); \
                      ({tuple_name}){{ .v0 = __sb, .v1 = __sa }}; }})"
            )
        }

        // Immediately-invoked lambda; captures by value, returns std::pair.
        Backend::Cpp => format!(
            "[&]() {{ auto __sa = {a}; auto __sb = {b}; \
               return std::make_pair(__sb, __sa); }}()"
        ),

        // Same root problem and fix as the C arm above: needs a's concrete
        // type to know which Tuple_T_T struct the calling function actually
        // declares as its return type (codegen_go.rs's tuple_go_name). The
        // old form returned a native Go two-value return `(interface{},
        // interface{})`, which doesn't compile against a function declared
        // to return the named Tuple_T_T struct at all — confirmed with a
        // real go build ("too many return values").
        Backend::Go => {
            let ty = p_types[0];
            if matches!(ty, bullang::ast::BuType::Unknown) {
                return Err(format!(
                    "'builtin::swap' (Go backend) couldn't determine the type of '{a}' — \
                     this needs a concrete type to build the returned Tuple[T,T] value"
                ));
            }
            let tuple_name = crate::codegen::codegen_go::tuple_go_name(&[ty.clone(), ty.clone()]);
            format!(
                "func() {tuple_name} {{ __sa := {a}; __sb := {b}; \
                   return {tuple_name}{{V0: __sb, V1: __sa}} }}()"
            )
        }

        Backend::Java    => format!(
            "((java.util.function.Supplier<Object[]>)(() -> {{ \
               var __sa = {a}; var __sb = {b}; \
               return new Object[]{{__sb, __sa}}; \
             }})).get()",
            a = a, b = b
        ),
        Backend::Unknown(kw) => return Err(format!(
            "'builtin::swap' is not available for unknown backend '{kw}'"
        )),
    })
}
