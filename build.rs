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

        format!("{}-pre.{}", version, git_hash)
    };

    println!("cargo:rustc-env=APP_VERSION={}", full_version);
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-env-changed=RELEASE_BUILD");

    Ok(())
}
