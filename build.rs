use std::process::Command;
use std::{env, io};

use winresource::WindowsResource;

fn main() -> io::Result<()> {
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        WindowsResource::new()
            .set_icon("assets/icon.ico")
            .compile()?;
    }

    #[cfg(all(unix, feature = "static-libpcap"))]
    {
        println!("cargo:rustc-link-lib=static=pcap");
    }

    // Set version with git hash for pre-releases
    let version = env!("CARGO_PKG_VERSION");
    let is_release = env::var("RELEASE_BUILD").is_ok();
    let is_debug = env::var("PROFILE").map(|p| p == "debug").unwrap_or(false);

    let full_version = if is_release {
        version.to_string()
    } else {
        // Get git hash at build.rs runtime
        let git_hash = Command::new("git")
            .args(["rev-parse", "--short=7", "HEAD"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok()
                } else {
                    None
                }
            })
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        if is_debug {
            format!("{}-debug.{}", version, git_hash)
        } else {
            format!("{}-pre.{}", version, git_hash)
        }
    };

    println!("cargo:rustc-env=APP_VERSION={}", full_version);
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-env-changed=RELEASE_BUILD");

    // Extract GitHub repository info from git remote
    let repo_info = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .and_then(|url| {
            let url = url.trim();
            // Handle both HTTPS and SSH URLs
            // HTTPS: https://github.com/owner/repo.git
            // SSH: git@github.com:owner/repo.git
            url.strip_prefix("https://github.com/")
                .or_else(|| url.strip_prefix("git@github.com:"))
                .map(|rest| rest.trim_end_matches(".git").to_string())
        });

    if let Some(repo) = repo_info {
        let parts: Vec<&str> = repo.split('/').collect();
        if parts.len() == 2 {
            println!("cargo:rustc-env=GITHUB_REPO_OWNER={}", parts[0]);
            println!("cargo:rustc-env=GITHUB_REPO_NAME={}", parts[1]);
        }
    }

    Ok(())
}
