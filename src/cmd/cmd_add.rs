//! `add` — Bullarchy package manager.
//!
//! bullarchy add                  → list all known packages from the registry
//! bullarchy add <name>           → install latest tagged version of a package
//! bullarchy add <name>@<ver>     → install a specific version
//! bullarchy add <https://...>    → install directly from a git URL (latest tag)
//! bullarchy add <https://...>@v1 → install specific version from a git URL

use std::fs;
use std::path::PathBuf;
use std::process::Command;

// ── Constants ─────────────────────────────────────────────────────────────────

const REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/The-Bullang-Foundation/Bullarchy-registery/main/registry.json";

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn cmd_add(args: &[&str]) {
    if args.is_empty() {
        list_packages();
        return;
    }

    let raw = args[0];

    // Split on '@' for optional version pin: name@version or url@version
    let (source, version) = match raw.splitn(2, '@').collect::<Vec<_>>().as_slice() {
        [s, v] => (s.to_string(), Some(v.to_string())),
        [s]    => (s.to_string(), None),
        _      => (raw.to_string(), None),
    };

    if source.starts_with("https://") || source.starts_with("http://") {
        install_from_url(&source, version.as_deref(), None);
    } else {
        install_from_registry(&source, version.as_deref());
    }
}

// ── List ──────────────────────────────────────────────────────────────────────

fn list_packages() {
    println!("  Fetching package list...\n");

    let registry = match fetch_registry() {
        Some(r) => r,
        None => {
            eprintln!("  Could not reach the Bullarchy registry.");
            eprintln!("  Check your internet connection or visit:");
            eprintln!("  https://github.com/The-Bullang-Foundation/Bullarchy-registery");
            return;
        }
    };

    let packages = match registry.as_object() {
        Some(p) => p,
        None => { eprintln!("  Registry format error."); return; }
    };

    if packages.is_empty() {
        println!("  No packages available yet.");
        return;
    }

    println!("  Available packages:\n");
    let max_name = packages.keys().map(|k| k.len()).max().unwrap_or(0);

    for (name, meta) in packages {
        let description = meta["description"].as_str().unwrap_or("No description.");
        let version     = meta["version"].as_str().unwrap_or("?");
        println!(
            "    {:<width$}  {}  — {}",
            name, version, description,
            width = max_name
        );
    }

    println!();
    println!("  Install with:  add <name>");
    println!("  Version pin:   add <name>@<version>");
    println!("  From URL:      add <https://github.com/...>");
    println!();
}

// ── Install from registry ─────────────────────────────────────────────────────

fn install_from_registry(name: &str, version: Option<&str>) {
    let registry = match fetch_registry() {
        Some(r) => r,
        None => {
            eprintln!("  Could not reach the Bullarchy registry.");
            return;
        }
    };

    let meta = match registry.get(name) {
        Some(m) => m,
        None => {
            eprintln!("  Unknown package '{}'. Run 'add' to see available packages.", name);
            return;
        }
    };

    let git_url = match meta["git"].as_str() {
        Some(u) => u.to_string(),
        None    => { eprintln!("  Registry entry for '{}' has no git URL.", name); return; }
    };

    install_from_url(&git_url, version, Some(name));
}

// ── Install from URL ──────────────────────────────────────────────────────────

fn install_from_url(git_url: &str, version: Option<&str>, package_name: Option<&str>) {
    // Derive a local package name from the URL if not provided
    let name = package_name.map(|s| s.to_string()).unwrap_or_else(|| {
        git_url
            .trim_end_matches('/')
            .trim_end_matches(".git")
            .split('/')
            .last()
            .unwrap_or("unknown")
            .to_string()
    });

    // Resolve which version to install
    let tag = match version {
        Some(v) => v.to_string(),
        None    => match latest_tag(git_url) {
            Some(t) => t,
            None => {
                eprintln!("  Could not find any release tags in '{}'.", git_url);
                eprintln!("  Try: add {}@<version>", name);
                return;
            }
        }
    };

    let install_dir = packages_dir().join(&name).join(&tag);

    if install_dir.exists() {
        println!("  '{}' {} is already installed.", name, tag);
        return;
    }

    println!("  Installing {} {}...", name, tag);

    fs::create_dir_all(&install_dir)
        .expect("could not create package directory");

    // Clone at the specific tag, shallow for speed
    let status = Command::new("git")
        .args([
            "clone",
            "--depth", "1",
            "--branch", &tag,
            git_url,
            install_dir.to_str().unwrap(),
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            update_lockfile(&name, &tag, git_url);
            println!("  Installed {} {} → {}", name, tag, install_dir.display());
            println!();
            println!("  To use in a project, add to your inventory.bu:");
            println!("    #lib: {};", name);
        }
        Ok(s) => {
            let _ = fs::remove_dir_all(&install_dir);
            eprintln!("  git clone failed (exit {}).", s);
        }
        Err(e) => {
            let _ = fs::remove_dir_all(&install_dir);
            eprintln!("  Failed to run git: {}.", e);
        }
    }
}

// ── Registry fetching ─────────────────────────────────────────────────────────

fn fetch_registry() -> Option<serde_json::Value> {
    let output = Command::new("curl")
        .args(["-sf", "--max-time", "10", REGISTRY_URL])
        .output()
        .ok()?;

    if !output.status.success() { return None; }

    let text = String::from_utf8(output.stdout).ok()?;
    serde_json::from_str(&text).ok()
}

// ── Git helpers ───────────────────────────────────────────────────────────────

/// Returns the latest semver git tag from a remote repo (e.g. "v1.2.3").
/// Fetches all tags, filters those that match vX.Y.Z, returns the highest.
fn latest_tag(repo: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["ls-remote", "--tags", "--sort=-version:refname", repo])
        .output()
        .ok()?;

    let stdout = String::from_utf8(output.stdout).ok()?;

    stdout
        .lines()
        .filter_map(|line| {
            // Lines look like: <sha>\trefs/tags/v1.2.3
            // Skip ^{} peeled tag entries
            let tag_ref = line.split('\t').nth(1)?;
            if tag_ref.ends_with("^{}") { return None; }
            let tag = tag_ref.trim_start_matches("refs/tags/");
            if is_semver(tag) { Some(tag.to_string()) } else { None }
        })
        .next()
}

/// Basic semver check: vX, vX.Y, or vX.Y.Z where X/Y/Z are digits.
fn is_semver(s: &str) -> bool {
    let s = s.trim_start_matches('v');
    s.split('.').all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

// ── Lockfile ──────────────────────────────────────────────────────────────────

fn update_lockfile(name: &str, version: &str, git_url: &str) {
    let lock_path = bull_home().join("bull.lock");

    let mut lock: serde_json::Value = if lock_path.exists() {
        let content = fs::read_to_string(&lock_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    lock[name] = serde_json::json!({
        "version": version,
        "git":     git_url,
    });

    fs::write(&lock_path, serde_json::to_string_pretty(&lock).unwrap())
        .expect("could not write bull.lock");
}

// ── Paths ─────────────────────────────────────────────────────────────────────

pub fn bull_home() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir  = PathBuf::from(home).join(".bull");
    fs::create_dir_all(&dir).ok();
    dir
}

pub fn packages_dir() -> PathBuf {
    let dir = bull_home().join("packages");
    fs::create_dir_all(&dir).ok();
    dir
}
