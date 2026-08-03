use bullang::ast::{Backend, Param};

pub const META: (&str, &str, &str) = (
    "out",
    "(fd: i32, content: String) → i32",
    "Write a string to a file descriptor. Returns bytes written, -1 on error (Java: only fd 1/stdout and 2/stderr are supported — needs JNI for arbitrary fds)",
);

// File name is fd_out.rs to stay consistent with fd_in.rs naming convention.
// The builtin name in Bullang source is `builtin::out`.

pub fn emit(params: &[Param], backend: &Backend) -> Result<String, String> {
    let p = super::need("out", params, 2)?;
    let (fd, content) = (p[0], p[1]);

    Ok(match backend {
        // ── Rust ─────────────────────────────────────────────────────────────
        // Wraps the raw fd in a ManuallyDrop<File> so we can write without
        // the File destructor closing the fd on drop.
        Backend::Rust => format!(
            "{{\
               let mut __f = ManuallyDrop::new(unsafe {{ \
                 std::fs::File::from_raw_fd({fd}) \
               }});\
               let __bytes = {content}.as_bytes();\
               __f.write_all(__bytes).map(|_| __bytes.len() as i32).unwrap_or(-1)\
             }}"
        ),

        // ── Python ───────────────────────────────────────────────────────────
        // os.write returns bytes written directly.
        Backend::Python => {
            let fd = super::py_esc(fd);
            let content = super::py_esc(content);
            format!(
                "(lambda __os, __fd, __b: __os.write(__fd, __b))\
                 (__import__('os'), {fd}, {content}.encode('utf-8'))"
            )
        }

        // ── C ────────────────────────────────────────────────────────────────
        // write(2); returns ssize_t. No cast needed — nothing downstream
        // requires exactly int32_t, and a bare call (unlike a cast used as
        // a discarded statement) never trips -Wunused-value either way.
        Backend::C => format!(
            "write({fd}, {content}, strlen({content}))"
        ),

        // ── C++ ──────────────────────────────────────────────────────────────
        Backend::Cpp => format!(
            "write({fd}, {content}.c_str(), {content}.size())"
        ),

        // ── Go ───────────────────────────────────────────────────────────────
        // syscall.Write returns (n int, err error).
        Backend::Go => format!(
            "func() int32 {{ \
               __b := []byte({content}); \
               __n, __err := syscall.Write(int({fd}), __b); \
               if __err != nil {{ return -1 }} \
               return int32(__n); \
             }}()"
        ),

        // ── Java ─────────────────────────────────────────────────────────────
        // Was previously broken: ignored the fd parameter entirely and
        // always wrote to System.out regardless of what was passed. Java
        // has no idiomatic public API for arbitrary raw OS file
        // descriptors, so this now only supports the conventional fds 1/2
        // (stdout/stderr), handled the fully idiomatic way via
        // System.out/System.err — any other fd is honestly unsupported
        // (returns -1) rather than silently writing to stdout anyway.
        // Real arbitrary-fd support needs a JNI native-library follow-up.
        Backend::Java => format!(
            "((java.util.function.IntSupplier)(() -> {{ \
               java.io.PrintStream __out; \
               if (({fd}) == 1) __out = System.out; \
               else if (({fd}) == 2) __out = System.err; \
               else return -1; \
               try {{ \
                 byte[] __b = {content}.getBytes(java.nio.charset.StandardCharsets.UTF_8); \
                 __out.write(__b); \
                 __out.flush(); \
                 return __b.length; \
               }} catch (java.io.IOException __e) {{ return -1; }} \
             }})).getAsInt()"
        ),
        Backend::Unknown(kw) => return Err(format!(
            "'builtin::out' is not available for unknown backend '{kw}'"
        )),
    })
}
