//! HTTP route handlers for all Bullarchy commands.
//!
//! Each handler redirects stdout/stderr to a buffer so the GUI can
//! display the output in the browser console panel.

use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ── Response type ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct CommandResult {
    pub ok:     bool,
    pub output: String,
}

fn ok(output: String) -> Json<CommandResult> {
    Json(CommandResult { ok: true, output })
}

// ── Output capture ────────────────────────────────────────────────────────────
//
// The cmd_* functions print to stdout/stderr directly.
// We redirect fd 1 & 2 via a pipe for the duration of the call.

fn capture<F: FnOnce()>(f: F) -> String {
    // Redirect stdout + stderr to a pipe
    let (reader, writer) = os_pipe::pipe().unwrap();
    let writer2 = writer.try_clone().unwrap();

    let old_stdout = redirect_fd(1, &writer);
    let old_stderr = redirect_fd(2, &writer2);

    // Drop our write ends so EOF is sent when the cmd finishes
    drop(writer);
    drop(writer2);

    // Run the command
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).ok();

    // Restore fds
    restore_fd(1, old_stdout);
    restore_fd(2, old_stderr);

    // Read captured output
    let mut buf = String::new();
    use std::io::Read;
    let mut r = reader;
    let _ = r.read_to_string(&mut buf);
    buf
}

#[cfg(unix)]
fn redirect_fd(fd: i32, writer: &os_pipe::PipeWriter) -> i32 {
    use std::os::unix::io::AsRawFd;
    unsafe {
        let old = libc::dup(fd);
        libc::dup2(writer.as_raw_fd(), fd);
        old
    }
}

#[cfg(unix)]
fn restore_fd(fd: i32, old: i32) {
    unsafe { libc::dup2(old, fd); libc::close(old); }
}

#[cfg(windows)]
fn redirect_fd(_fd: i32, _writer: &os_pipe::PipeWriter) -> i32 { -1 }
#[cfg(windows)]
fn restore_fd(_fd: i32, _old: i32) {}

// ── /api/init ─────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct InitRequest {
    pub name:      String,
    pub depth:     Option<u8>,
    pub lang:      Option<String>,
    pub libs:      Option<Vec<String>>,
    pub blueprint: Option<String>,
    pub path:      Option<String>,
}

pub async fn handle_init(Json(req): Json<InitRequest>) -> impl IntoResponse {
    let name      = req.name.clone();
    let depth     = req.depth.unwrap_or(2);
    let lang      = req.lang.clone();
    let libs      = req.libs.unwrap_or_default();
    let blueprint = req.blueprint.map(PathBuf::from);
    let path      = req.path.map(PathBuf::from);

    let output = capture(move || {
        crate::cmd::cmd_init(name, depth, blueprint, lang, libs, path);
    });

    ok(output)
}

// ── /api/convert ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ConvertRequest {
    pub target: Option<String>,
    pub second: Option<String>,
}

pub async fn handle_convert(Json(req): Json<ConvertRequest>) -> impl IntoResponse {
    let target = req.target.map(PathBuf::from);
    let second = req.second.clone();

    let output = capture(move || {
        crate::cmd::cmd_convert(target, second);
    });

    ok(output)
}

// ── /api/fmt ──────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct FmtRequest {
    pub folder:  Option<String>,
    pub dry_run: Option<bool>,
}

pub async fn handle_fmt(Json(req): Json<FmtRequest>) -> impl IntoResponse {
    let folder  = req.folder.map(PathBuf::from);
    let dry_run = req.dry_run.unwrap_or(false);

    let output = capture(move || {
        crate::cmd::cmd_fmt(folder, dry_run);
    });

    ok(output)
}

// ── /api/check ────────────────────────────────────────────────────────────────

pub async fn handle_check() -> impl IntoResponse {
    let output = capture(|| {
        crate::cmd::cmd_check();
    });

    ok(output)
}

// ── /api/editor-setup ─────────────────────────────────────────────────────────

pub async fn handle_editor_setup() -> impl IntoResponse {
    let output = capture(|| {
        crate::cmd::cmd_editor_setup();
    });

    ok(output)
}

// ── /api/update ───────────────────────────────────────────────────────────────

pub async fn handle_update() -> impl IntoResponse {
    let output = capture(|| {
        crate::cmd::cmd_update();
    });

    ok(output)
}

// ── /api/blueprint/save ───────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct BlueprintSaveRequest {
    pub path:    String,
    pub content: String,
}

#[derive(Serialize)]
pub struct SaveResult {
    pub ok:    bool,
    pub error: Option<String>,
}

pub async fn handle_blueprint_save(Json(req): Json<BlueprintSaveRequest>) -> impl IntoResponse {
    let path = std::path::PathBuf::from(&req.path);

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return Json(SaveResult { ok: false, error: Some(e.to_string()) });
        }
    }

    match std::fs::write(&path, &req.content) {
        Ok(_)  => Json(SaveResult { ok: true, error: None }),
        Err(e) => Json(SaveResult { ok: false, error: Some(e.to_string()) }),
    }
}
