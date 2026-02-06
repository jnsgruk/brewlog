use std::process::Command;

fn main() {
    git_hash();
    tailwind();
}

fn git_hash() {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    let hash = output.trim();
    println!("cargo:rustc-env=GIT_HASH={hash}");
}

fn tailwind() {
    let mut cmd = Command::new("tailwindcss");
    cmd.args(["-i", "static/css/input.css", "-o", "static/css/styles.css"]);

    if std::env::var("PROFILE").as_deref() == Ok("release") {
        cmd.arg("--minify");
    }

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
}
