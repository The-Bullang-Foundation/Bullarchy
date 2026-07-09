use bullang::ast::{Backend, Param};

pub const META: (&str, &str, &str) = (
    "run",
    "(cmd: String)             → i32",
    "Run a shell command. Returns exit code (0 = success, non-zero = failure)",
);

// Shell dispatch:
//   Linux / macOS → sh -c "<cmd>"
//   Windows       → cmd /C "<cmd>"
//
// The OS is detected at runtime in the generated code, not at transpile time,
// so the same emitted code works on all platforms without recompilation.

pub fn emit(params: &[Param], backend: &Backend) -> Result<String, String> {
    let p = super::need("run", params, 1)?;
    let cmd = p[0];

    Ok(match backend {
        // ── Rust ─────────────────────────────────────────────────────────────
        // std::process::Command is fully cross-platform.
        // On Unix we use "sh -c", on Windows "cmd /C".
        Backend::Rust => format!(
            "{{\
               let __cmd = {cmd};\
               let __status = if cfg!(target_os = \"windows\") {{\
                 std::process::Command::new(\"cmd\")\
                   .args([\"/C\", __cmd.as_str()])\
                   .status()\
               }} else {{\
                 std::process::Command::new(\"sh\")\
                   .args([\"-c\", __cmd.as_str()])\
                   .status()\
               }};\
               __status.map(|s| s.code().unwrap_or(-1)).unwrap_or(-1)\
             }}"
        ),

        // ── Python ───────────────────────────────────────────────────────────
        // subprocess.call is cross-platform; shell=True lets the OS pick
        // the right shell automatically.
        Backend::Python => {
            let cmd = super::py_esc(cmd);
            format!(
                "__import__('subprocess').call({cmd}, shell=True)"
            )
        }

        // ── C ────────────────────────────────────────────────────────────────
        // system(3) is defined in <stdlib.h> and is cross-platform (C99).
        // Returns the exit code on POSIX; on Windows it returns the exit
        // code of cmd.exe, which is the program's exit code for simple calls.
        Backend::C => format!("system({cmd})"),

        // ── C++ ──────────────────────────────────────────────────────────────
        // Same as C — std::system is in <cstdlib>.
        Backend::Cpp => format!("std::system({cmd}.c_str())"),

        // ── Go ───────────────────────────────────────────────────────────────
        // exec.Command + runtime.GOOS for shell detection.
        // Returns the exit code as int32.
        Backend::Go => format!(
            "func() int32 {{\
               var __c *exec.Cmd;\
               if runtime.GOOS == \"windows\" {{\
                 __c = exec.Command(\"cmd\", \"/C\", {cmd});\
               }} else {{\
                 __c = exec.Command(\"sh\", \"-c\", {cmd});\
               }}\
               if __err := __c.Run(); __err != nil {{\
                 if __ee, __ok := __err.(*exec.ExitError); __ok {{\
                   return int32(__ee.ExitCode());\
                 }}\
                 return -1;\
               }}\
               return 0;\
             }}()"
        ),

        // ── Java ─────────────────────────────────────────────────────────────
        // ProcessBuilder is cross-platform; os.name detects Windows.
        Backend::Java => format!(
            "((java.util.function.IntSupplier)(() -> {{\
               try {{\
                 String[] __sh = System.getProperty(\"os.name\", \"\").toLowerCase().contains(\"win\")\
                   ? new String[]{{\"cmd\", \"/C\", {cmd}}}\
                   : new String[]{{\"sh\", \"-c\", {cmd}}};\
                 return new ProcessBuilder(__sh)\
                   .inheritIO()\
                   .start()\
                   .waitFor();\
               }} catch (Exception __e) {{ return -1; }}\
             }})).getAsInt()"
        ),

        Backend::Unknown(kw) => return Err(format!(
            "'builtin::run' is not available for unknown backend '{kw}'"
        )),
    })
}
