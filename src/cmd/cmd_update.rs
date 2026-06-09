//! `update` — reinstall Bullarchy from the source repository.

pub const DEFAULT_REPO: &str = "https://github.com/My-sidequests/Bullarchy.git";

pub fn cmd_update() {
    println!("Updating bullarchy...");

    let remote = match remote_head(DEFAULT_REPO, "main") {
        Some(h) => h,
        None => {
            eprintln!("Could not reach repository. Check your internet connection.");
            return;
        }
    };

    let installed = installed_hash("bullarchy", DEFAULT_REPO, "main");

    if installed.map_or(false, |h| remote.starts_with(&h)) {
        println!("Already up to date (commit: {}).", &remote[..8]);
        return;
    }

    let status = std::process::Command::new("cargo")
        .args(["install", "--git", DEFAULT_REPO, "--branch", "main", "--force", "bullarchy"])
        .status();

    match status {
        Ok(s) if s.success() => println!("Update complete."),
        Ok(s)  => eprintln!("cargo install exited with {}.", s),
        Err(e) => eprintln!("Failed to run cargo: {}.", e),
    }
}

/// Fetch the HEAD commit hash of `branch` from a remote git repository.
/// Returns the full 40-character SHA, or None if git is unavailable or the
/// repo cannot be reached.
fn remote_head(repo: &str, branch: &str) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["ls-remote", repo, &format!("refs/heads/{}", branch)])
        .output()
        .ok()?;

    let stdout = String::from_utf8(output.stdout).ok()?;
    let hash = stdout.split_whitespace().next()?;
    if hash.len() == 40 { Some(hash.to_string()) } else { None }
}

/// Read the commit hash for `package` as recorded in ~/.cargo/.crates2.json.
/// Returns the short hash stored by cargo (e.g. "aaec925f"), or None if not
/// found or the file cannot be parsed.
fn installed_hash(package: &str, repo: &str, branch: &str) -> Option<String> {
    let cargo_home = std::env::var("CARGO_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            std::path::PathBuf::from(home).join(".cargo")
        });

    let crates2 = std::fs::read_to_string(cargo_home.join(".crates2.json")).ok()?;

    let branch_tag = format!("branch={}", branch);
    let repo_fragment = repo.trim_end_matches(".git");

    for line in crates2.lines() {
        if line.contains(package)
            && line.contains(repo_fragment)
            && line.contains(&branch_tag)
        {
            if let Some(hash_start) = line.rfind('#') {
                let rest = &line[hash_start + 1..];
                if let Some(hash_end) = rest.find('"') {
                    return Some(rest[..hash_end].to_string());
                }
            }
        }
    }
    None
}
