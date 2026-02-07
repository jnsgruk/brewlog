use std::path::Path;
use std::process::Command;

fn main() {
    git_hash();
    tailwind();
}

fn git_hash() {
    let hash = std::env::var("GIT_HASH").ok().or_else(|| {
        Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| String::from_utf8(o.stdout).ok())
    });
    let hash = hash.as_deref().map(str::trim).unwrap_or_default();
    println!("cargo:rustc-env=GIT_HASH={hash}");
}

fn tailwind() {
    println!("cargo:rerun-if-changed=static/css/input.css");
    println!("cargo:rerun-if-changed=templates/");

    let mut cmd = Command::new("tailwindcss");
    cmd.args(["-i", "static/css/input.css", "-o", "static/css/styles.css"]);

    if std::env::var("PROFILE").as_deref() == Ok("release") {
        cmd.arg("--minify");
    }

    let output = Path::new("static/css/styles.css");

    match cmd.status() {
        Ok(status) if status.success() => {}
        Ok(status) => {
            eprintln!("cargo:warning=tailwindcss exited with {status}");
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("cargo:warning=tailwindcss not found on PATH, skipping CSS generation");
        }
        Err(e) => {
            eprintln!("cargo:warning=failed to run tailwindcss: {e}");
        }
    }

    // Ensure the file exists so include_str!() doesn't break the build
    if !output.exists() {
        std::fs::write(output, "/* tailwindcss not available */").ok();
    }
}
