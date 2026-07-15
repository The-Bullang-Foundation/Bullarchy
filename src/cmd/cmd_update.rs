//! `update` — clean and reinstall the full Bullang ecosystem.
//!
//! Steps:
//!   1. Clear cargo git cache for all Bullang repos
//!   2. cargo install --force for Bullang, Bullarchy, Bullscript

use std::path::PathBuf;

pub const DEFAULT_REPO: &str = "https://github.com/The-Bullang-Foundation/Bullarchy.git";

const BULLANG_REPO:    &str = "https://github.com/The-Bullang-Foundation/Bullang.git";
const BULLARCHY_REPO:  &str = "https://github.com/The-Bullang-Foundation/Bullarchy.git";
const BULLSCRIPT_REPO: &str = "https://github.com/The-Bullang-Foundation/Bullscript.git";

pub fn cmd_update() {
    println!("  Updating Bullang ecosystem...\n");

    // ── Step 1: Clear cargo git caches ───────────────────────────────────────

    println!("  Clearing cargo git cache...");
    let cargo_home = cargo_home();

    for dir in &["bullang", "bullarchy", "bullscript"] {
        for subfolder in &["checkouts", "db"] {
            let pattern = cargo_home.join("git").join(subfolder);
            if let Ok(entries) = std::fs::read_dir(&pattern) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_lowercase();
                    if name.contains(dir) {
                        let path = entry.path();
                        match std::fs::remove_dir_all(&path) {
                            Ok(_)  => println!("    removed {}", path.display()),
                            Err(e) => eprintln!("    warning: could not remove {}: {}", path.display(), e),
                        }
                    }
                }
            }
        }
    }

    // Also remove old installed binaries so cargo reinstalls cleanly
    for bin in &["bullang", "bullarchy", "bullscript"] {
        let bin_path = cargo_home.join("bin").join(bin);
        if bin_path.exists() {
            let _ = std::fs::remove_file(&bin_path);
            println!("    removed binary: {}", bin_path.display());
        }
        // Windows
        let bin_exe = cargo_home.join("bin").join(format!("{}.exe", bin));
        if bin_exe.exists() {
            let _ = std::fs::remove_file(&bin_exe);
            println!("    removed binary: {}", bin_exe.display());
        }
    }

    println!();

    // ── Step 2: Reinstall all three ───────────────────────────────────────────

    let installs = [
        ("Bullang",    BULLANG_REPO,    "bullang"),
        ("Bullarchy",  BULLARCHY_REPO,  "bullarchy"),
        ("Bullscript", BULLSCRIPT_REPO, "bullscript"),
    ];

    let mut failed = false;

    for (name, repo, bin) in &installs {
        println!("  Installing {} from {}...", name, repo);
        let status = std::process::Command::new("cargo")
            .args(["install", "--git", repo, "--force", bin])
            .status();

        match status {
            Ok(s) if s.success() => println!("  ✓ {} updated.\n", name),
            Ok(s) => {
                eprintln!("  ✗ {} failed (exit {}).\n", name, s);
                failed = true;
            }
            Err(e) => {
                eprintln!("  ✗ Failed to run cargo for {}: {}.\n", name, e);
                failed = true;
            }
        }
    }

    if failed {
        eprintln!("  Some updates failed. Check the output above.");
    } else {
        println!("  ✓ All Bullang tools updated successfully.");
        println!("  Restart your terminal to ensure the new binaries are active.");
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn cargo_home() -> PathBuf {
    std::env::var("CARGO_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_default();
            PathBuf::from(home).join(".cargo")
        })
}

/// Fetch the HEAD commit hash of `branch` from a remote git repository.
pub fn remote_head(repo: &str, branch: &str) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["ls-remote", repo, &format!("refs/heads/{}", branch)])
        .output()
        .ok()?;

    let stdout = String::from_utf8(output.stdout).ok()?;
    let hash = stdout.split_whitespace().next()?;
    if hash.len() == 40 { Some(hash.to_string()) } else { None }
}

/// Read the commit hash for `package` as recorded in ~/.cargo/.crates2.json.
pub fn installed_hash(package: &str, repo: &str, branch: &str) -> Option<String> {
    let content = std::fs::read_to_string(
        cargo_home().join(".crates2.json")
    ).ok()?;

    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let installs = json["installs"].as_object()?;

    let repo_fragment = repo.trim_end_matches(".git");
    let branch_tag = format!("branch={}", branch);

    for key in installs.keys() {
        if key.contains(package)
            && key.contains(repo_fragment)
            && key.contains(&branch_tag)
        {
            let hash = key.split('#').nth(1)?.trim_end_matches(')');
            return Some(hash.to_string());
        }
    }
    None
}
