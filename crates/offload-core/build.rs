use std::path::{Path, PathBuf};
use std::process::Command;

fn git_output(repo: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn main() {
    println!("cargo:rerun-if-env-changed=GIT_COMMIT");
    println!("cargo:rerun-if-env-changed=GITHUB_SHA");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=../../src-tauri/src/offload");

    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let repo = git_output(&manifest_dir, &["rev-parse", "--show-toplevel"])
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.clone());

    let explicit = std::env::var("GIT_COMMIT")
        .ok()
        .or_else(|| std::env::var("GITHUB_SHA").ok());
    let mut commit = explicit
        .or_else(|| git_output(&repo, &["rev-parse", "HEAD"]))
        .unwrap_or_else(|| "unknown".to_string());

    if commit != "unknown"
        && git_output(&repo, &["status", "--porcelain", "--untracked-files=no"])
            .is_some_and(|status| !status.is_empty())
    {
        commit.push_str("-dirty");
    }

    if let Some(git_dir) = git_output(&repo, &["rev-parse", "--git-dir"]) {
        let git_dir = {
            let path = PathBuf::from(git_dir);
            if path.is_absolute() {
                path
            } else {
                repo.join(path)
            }
        };
        println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());
        if let Ok(head) = std::fs::read_to_string(git_dir.join("HEAD")) {
            if let Some(reference) = head.strip_prefix("ref: ").map(str::trim) {
                println!(
                    "cargo:rerun-if-changed={}",
                    git_dir.join(reference).display()
                );
            }
        }
    }

    println!("cargo:rustc-env=GIT_COMMIT={commit}");
}
