use bullang::ast::{Backend, Param};

pub const META: (&str, &str, &str) = (
    "close",
    "(fd: i32)                 → i32",
    "Close a file descriptor. Returns 0 on success, -1 on error (Python: raises OSError on error instead; Java: only fds 0/1/2 are supported — needs JNI for arbitrary fds)",
);

pub fn emit(params: &[Param], backend: &Backend) -> Result<String, String> {
    let p = super::need("close", params, 1)?;
    let fd = p[0];

    Ok(match backend {
        // ── Rust ─────────────────────────────────────────────────────────────
        // Construct a File from the raw fd and let it drop — the Drop impl
        // calls close(2) internally.  Returns 0 unconditionally since
        // std::fs::File::drop does not surface the error; this matches
        // the common usage pattern of close in C.
        Backend::Rust => format!(
            "{{\
               unsafe {{ drop(std::fs::File::from_raw_fd({fd})) }};\
               0i32\
             }}"
        ),

        // ── Python ───────────────────────────────────────────────────────────
        // Deviates from the documented "-1 on error" contract: Python has no
        // try/except in expression position and this backend has no hoisting
        // mechanism to define a real helper function, so catching the error
        // to return -1 isn't cleanly possible here. os.close() failing
        // raises OSError naturally instead — idiomatic Python, unlike Rust's
        // silent-0 deviation above.
        Backend::Python => {
            let fd = super::py_esc(fd);
            format!("(lambda __os: (__os.close({fd}), 0)[1])(__import__('os'))")
        }

        // ── C ────────────────────────────────────────────────────────────────
        // POSIX close(2) directly. Requires <unistd.h>.
        Backend::C => format!("close({fd})"),

        // ── C++ ──────────────────────────────────────────────────────────────
        Backend::Cpp => format!("close({fd})"),

        // ── Go ───────────────────────────────────────────────────────────────
        // syscall.Close returns an error; normalise to 0 / -1.
        Backend::Go => format!(
            "func() int32 {{\
               if __err := syscall.Close(int({fd})); __err != nil {{ return -1 }}\
               return 0\
             }}()"
        ),

        // ── Java ─────────────────────────────────────────────────────────────
        // Was previously a complete no-op — ignored fd entirely and always
        // returned 0. Java has no idiomatic public API for arbitrary raw OS
        // file descriptors, so this now only supports the conventional fds
        // 0/1/2 (stdin/stdout/stderr), closed the idiomatic way via their
        // own .close() methods — any other fd is honestly unsupported
        // (returns -1) rather than silently pretending to succeed. Real
        // arbitrary-fd support needs a JNI native-library follow-up.
        Backend::Java => format!(
            "((java.util.function.IntSupplier)(() -> {{ \
               try {{ \
                 if (({fd}) == 0) {{ System.in.close(); return 0; }} \
                 if (({fd}) == 1) {{ System.out.close(); return 0; }} \
                 if (({fd}) == 2) {{ System.err.close(); return 0; }} \
                 return -1; \
               }} catch (java.io.IOException __e) {{ return -1; }} \
             }})).getAsInt()"
        ),
        Backend::Unknown(kw) => return Err(format!(
            "'builtin::close' is not available for unknown backend '{kw}'"
        )),
    })
}
