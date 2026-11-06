use std::process::Command;

fn main() {
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .or_else(|| std::env::var("TITAN_GIT_HASH").ok())
        .unwrap_or_else(|| "unknown".to_string());

    let build_date = std::env::var("TITAN_BUILD_DATE").unwrap_or_else(|_| {
        chrono_fallback_now()
    });

    println!("cargo:rustc-env=TITAN_GIT_HASH={}", git_hash);
    println!("cargo:rustc-env=TITAN_BUILD_DATE={}", build_date);

    tauri_build::build()
}

fn chrono_fallback_now() -> String {
    let output = Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok();
    if let Some(o) = output {
        if o.status.success() {
            if let Ok(s) = String::from_utf8(o.stdout) {
                return s.trim().to_string();
            }
        }
    }
    "unknown".to_string()
}
