use bullang::ast::{Backend, Param};

pub const META: (&str, &str, &str) = (
    "close",
    "(fd: i32)                 → i32",
    "Close a file descriptor. Returns 0 on success, -1 on error (Python: raises OSError on error instead; Java: fds 0/1/2 use System.in/out/err directly, other fds delegate to the JNI BuNative library)",
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
        // returned 0. The conventional fds 0/1/2 (stdin/stdout/stderr) keep
        // the fast, zero-setup idiomatic path via their own .close()
        // methods — no native library needed just to close stdio. Any other
        // fd falls through to BuNative (JNI), generated automatically
        // whenever this builtin is used — see codegen_jni.rs.
        //
        // When fd's source text is literally 0/1/2 (not just a variable
        // that happens to hold 0/1/2 at runtime), the branching/BuNative
        // fallback below is skipped entirely and replaced with a direct,
        // single-target call. This isn't just an optimization: a project
        // that never calls builtin::open (so codegen_jni.rs never emits
        // BuNative.java) would otherwise still reference a BuNative class
        // that doesn't exist, in dead-but-still-compiled code, for
        // something as simple as `builtin::close(1)`. System.out/err's
        // PrintStream.close() also doesn't declare `throws IOException`
        // (unlike System.in), so wrapping it in a try/catch there would
        // itself be a compile error ("exception is never thrown") — hence
        // the three literal cases aren't just one shared template.
        Backend::Java if fd.trim() == "0" => format!(
            "((java.util.function.IntSupplier)(() -> {{ \
               try {{ System.in.close(); return 0; }} \
               catch (java.io.IOException __e) {{ return -1; }} \
             }})).getAsInt()"
        ),
        Backend::Java if fd.trim() == "1" => "((java.util.function.IntSupplier)(() -> { \
               System.out.close(); return 0; \
             })).getAsInt()".to_string(),
        Backend::Java if fd.trim() == "2" => "((java.util.function.IntSupplier)(() -> { \
               System.err.close(); return 0; \
             })).getAsInt()".to_string(),
        Backend::Java => format!(
            "((java.util.function.IntSupplier)(() -> {{ \
               try {{ \
                 if (({fd}) == 0) {{ System.in.close(); return 0; }} \
                 if (({fd}) == 1) {{ System.out.close(); return 0; }} \
                 if (({fd}) == 2) {{ System.err.close(); return 0; }} \
                 return BuNative.nClose({fd}); \
               }} catch (java.io.IOException __e) {{ return -1; }} \
             }})).getAsInt()"
        ),
        Backend::Unknown(kw) => return Err(format!(
            "'builtin::close' is not available for unknown backend '{kw}'"
        )),
    })
}
