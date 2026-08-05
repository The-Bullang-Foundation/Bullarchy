use bullang::ast::{Backend, Param};

pub const META: (&str, &str, &str) = (
    "out",
    "(fd: i32, content: String) → i32",
    "Write a string to a file descriptor. Returns bytes written, -1 on error (Java: fds 1/2 use System.out/err directly, other fds delegate to the JNI BuNative library)",
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
        // always wrote to System.out regardless of what was passed. The
        // conventional fds 1/2 (stdout/stderr) keep the fast, zero-setup
        // idiomatic path via System.out/System.err — no native library
        // needed just to print. Any other fd falls through to BuNative
        // (JNI), generated automatically whenever this builtin is used —
        // see codegen_jni.rs.
        //
        // When fd's source text is literally "1" or "2" (not just a
        // variable that happens to hold that value at runtime), the
        // BuNative branch is dropped entirely rather than left in as dead
        // code — a project that never calls builtin::open won't have
        // BuNative.java generated at all (see codegen_jni.rs), so a stray
        // reference to it in unreachable code would still fail to compile.
        // (The try/catch stays either way: PrintStream inherits the 1-arg
        // write(byte[]) — the one used below — from OutputStream, which
        // does declare `throws IOException`, unlike close() a few lines up
        // in close.rs, which PrintStream overrides without that clause.)
        Backend::Java if fd.trim() == "1" || fd.trim() == "2" => {
            let stream = if fd.trim() == "1" { "System.out" } else { "System.err" };
            format!(
                "((java.util.function.IntSupplier)(() -> {{ \
                   try {{ \
                     byte[] __b = {content}.getBytes(java.nio.charset.StandardCharsets.UTF_8); \
                     {stream}.write(__b); \
                     {stream}.flush(); \
                     return __b.length; \
                   }} catch (java.io.IOException __e) {{ return -1; }} \
                 }})).getAsInt()"
            )
        }
        Backend::Java => format!(
            "((java.util.function.IntSupplier)(() -> {{ \
               java.io.PrintStream __out; \
               if (({fd}) == 1) __out = System.out; \
               else if (({fd}) == 2) __out = System.err; \
               else return BuNative.nOut({fd}, {content}); \
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
