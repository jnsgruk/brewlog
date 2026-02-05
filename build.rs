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
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads/");
}

fn tailwind() {
    // Rerun when any template file changes (Tailwind scans them for classes)
    for entry in walkdir("templates") {
        println!("cargo:rerun-if-changed={entry}");
    }

    let mut cmd = Command::new("tailwindcss");
    cmd.args(["-i", "templates/input.css", "-o", "templates/styles.css"]);

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

fn walkdir(dir: &str) -> Vec<String> {
    let mut files = Vec::new();
    let mut stack = vec![std::path::PathBuf::from(dir)];
    while let Some(path) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&path) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(path.display().to_string());
            }
        }
    }
    files
}
