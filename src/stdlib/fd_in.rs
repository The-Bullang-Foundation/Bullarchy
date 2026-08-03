use bullang::ast::{Backend, Param};

pub const META: (&str, &str, &str) = (
    "in",
    "(fd: i32)                 → String",
    "Read one line from a file descriptor (newline stripped). Empty string on EOF/error (Java: only fd 0/stdin is supported — needs JNI for arbitrary fds)",
);

// File name is fd_in.rs because `in` is a Rust reserved keyword and cannot
// be used as a module name.  The builtin name in Bullang source remains
// `builtin::in`.

pub fn emit(params: &[Param], backend: &Backend) -> Result<String, String> {
    let p = super::need("in", params, 1)?;
    let fd = p[0];

    Ok(match backend {
        // ── Rust ─────────────────────────────────────────────────────────────
        // Reads byte-by-byte, NOT via BufReader — a BufReader would fill its
        // internal buffer (~8KB) with a single read(2) syscall, silently
        // discarding anything past the first line when it's dropped at the
        // end of this block. Since a fresh reader is created on every
        // builtin::in() call (no persisted state), that would lose data on
        // any repeated call over the same fd — the normal "read lines in a
        // loop" usage. Byte-by-byte guarantees only the returned line's
        // bytes (plus the trailing newline) are ever consumed from the fd,
        // matching the C/C++ arms below.
        // The fd is NOT closed here — ownership stays with the caller.
        Backend::Rust => format!(
            "{{\
               let __fd = {fd};\
               let mut __f = unsafe {{ std::fs::File::from_raw_fd(__fd) }};\
               let mut __line = String::new();\
               let mut __byte = [0u8; 1];\
               loop {{\
                 match __f.read(&mut __byte) {{\
                   Ok(0) | Err(_) => break,\
                   Ok(_) => {{\
                     if __byte[0] == b'\\n' {{ break; }}\
                     __line.push(__byte[0] as char);\
                   }}\
                 }}\
               }}\
               std::mem::forget(__f);\
               __line.trim_end_matches('\\r').to_owned()\
             }}"
        ),

        // ── Python ───────────────────────────────────────────────────────────
        // iter(callable, sentinel) + itertools.takewhile are the native tools
        // for "read one at a time until a terminator" — iter(..., b'\n')
        // stops (without yielding it) at the newline; takewhile stops at EOF
        // (os.read returns b'' forever past EOF, which iter's sentinel check
        // alone wouldn't catch since it's only watching for b'\n').
        Backend::Python => {
            let fd = super::py_esc(fd);
            format!(
                "(lambda __os: \
                   b''.join(__import__('itertools').takewhile(\
                     lambda __b: __b != b'', \
                     iter(lambda: __os.read({fd}, 1), b'\\n')\
                   )).decode('utf-8', errors='replace').rstrip('\\r')\
                 )(__import__('os'))"
            )
        }

        // ── C ────────────────────────────────────────────────────────────────
        // read(2) byte-by-byte into a local buffer until '\\n' or EOF.
        // Returns a heap-allocated char* (caller owns it).
        Backend::C => format!(
            "({{ \
               char *__buf = (char *)malloc(4096); \
               size_t __i = 0; \
               char __ch; \
               ssize_t __n; \
               while (__i < 4095 && (__n = read({fd}, &__ch, 1)) > 0) {{ \
                 if (__ch == '\\n') break; \
                 __buf[__i++] = __ch; \
               }} \
               if (__i > 0 && __buf[__i-1] == '\\r') __i--; \
               __buf[__i] = '\\0'; \
               __buf; \
             }})"
        ),

        // ── C++ ──────────────────────────────────────────────────────────────
        // IIFE wrapping the same byte-by-byte read; returns std::string.
        Backend::Cpp => format!(
            "[&]() -> std::string {{ \
               std::string __s; \
               char __ch; \
               while (read({fd}, &__ch, 1) > 0) {{ \
                 if (__ch == '\\n') break; \
                 __s += __ch; \
               }} \
               if (!__s.empty() && __s.back() == '\\r') __s.pop_back(); \
               return __s; \
             }}()"
        ),

        // ── Go ───────────────────────────────────────────────────────────────
        // Wraps the raw fd in an os.File (without taking ownership via
        // runtime.SetFinalizer) then reads byte-by-byte via os.File.Read
        // directly — NOT bufio.Reader. bufio.Reader fills an internal
        // buffer (~4KB) with a single Read() syscall and, since a fresh
        // reader is created on every builtin::in() call (no persisted
        // state), silently discards anything past the first line when it
        // goes out of scope — losing data on repeated calls over the same
        // fd, the normal "read lines in a loop" usage. os.File.Read itself
        // does no such lookahead buffering, so byte-by-byte reads through
        // it directly are safe, matching the Rust arm's equivalent fix.
        Backend::Go => format!(
            "func() string {{ \
               __f := os.NewFile(uintptr({fd}), \"\"); \
               if __f == nil {{ return \"\" }} \
               var __sb strings.Builder; \
               __buf := make([]byte, 1); \
               for {{ \
                 __n, __err := __f.Read(__buf); \
                 if __n == 0 || __err != nil {{ break }} \
                 if __buf[0] == '\\n' {{ break }} \
                 __sb.WriteByte(__buf[0]); \
               }} \
               __line := strings.TrimRight(__sb.String(), \"\\r\"); \
               runtime.KeepAlive(__f); \
               return __line; \
             }}()"
        ),

        // ── Java ─────────────────────────────────────────────────────────────
        // Was previously broken: ignored the fd parameter entirely and
        // always read from java.io.FileDescriptor.in (stdin) regardless of
        // what was passed, via dead/unused reflection scaffolding that
        // wasn't even wired up. Java has no idiomatic public API for
        // arbitrary raw OS file descriptors, so this now only supports the
        // conventional fd 0 (stdin), handled the fully idiomatic way via
        // System.in — any other fd is honestly unsupported (returns "")
        // rather than silently reading stdin anyway. Real arbitrary-fd
        // support needs a JNI native-library follow-up.
        Backend::Java => format!(
            "((java.util.function.Supplier<String>)(() -> {{ \
               if (({fd}) != 0) return \"\"; \
               try {{ \
                 StringBuilder __sb = new StringBuilder(); \
                 int __ch; \
                 while ((__ch = System.in.read()) != -1) {{ \
                   if (__ch == '\\n') break; \
                   __sb.append((char) __ch); \
                 }} \
                 String __line = __sb.toString(); \
                 if (__line.endsWith(\"\\r\")) __line = __line.substring(0, __line.length() - 1); \
                 return __line; \
               }} catch (java.io.IOException __e) {{ return \"\"; }} \
             }})).get()"
        ),
        Backend::Unknown(kw) => return Err(format!(
            "'builtin::in' is not available for unknown backend '{kw}'"
        )),
    })
}
