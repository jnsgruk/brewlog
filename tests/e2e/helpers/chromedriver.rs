use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use once_cell::sync::Lazy;

unsafe extern "C" {
    fn prctl(option: i32, arg2: u64, arg3: u64, arg4: u64, arg5: u64) -> i32;
}
const PR_SET_PDEATHSIG: i32 = 1;
const SIGTERM: u64 = 15;

struct ChromedriverProcess {
    process: Child,
    port: u16,
}

impl Drop for ChromedriverProcess {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

/// Single shared chromedriver process for all e2e tests.
/// Spawned on first use, killed when the test binary exits via `PR_SET_PDEATHSIG`.
static CHROMEDRIVER: Lazy<Mutex<Option<ChromedriverProcess>>> = Lazy::new(|| Mutex::new(None));

/// Spawn chromedriver on a dedicated thread that stays alive for the lifetime of the process.
///
/// `PR_SET_PDEATHSIG` fires when the *thread* that called `fork()` exits, not when the
/// process exits. Since `ensure_chromedriver()` may be called from a short-lived tokio
/// worker thread, we spawn chromedriver from a dedicated parked thread to prevent
/// premature cleanup.
fn spawn_chromedriver(port: u16) -> Child {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);

    std::thread::spawn(move || {
        let mut cmd = Command::new("chromedriver");
        cmd.arg(format!("--port={port}"))
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        // SAFETY: prctl(PR_SET_PDEATHSIG, SIGTERM) is async-signal-safe.
        // It asks the kernel to send SIGTERM to the child when the parent exits,
        // preventing orphaned chromedriver processes.
        unsafe {
            cmd.pre_exec(|| {
                if prctl(PR_SET_PDEATHSIG, SIGTERM, 0, 0, 0) == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }

        let process = cmd
            .spawn()
            .expect("Failed to start chromedriver — is it installed?");

        let _ = tx.send(process);

        // Keep this thread alive so PR_SET_PDEATHSIG (bound to it) doesn't fire
        // until the test binary exits and all threads are torn down.
        loop {
            std::thread::park();
        }
    });

    rx.recv().expect("Chromedriver spawner thread panicked")
}

/// Ensure chromedriver is running and return its port.
///
/// If `CHROMEDRIVER_URL` is set (e.g. `http://localhost:4444`), assumes an external
/// chromedriver is already running and parses the port from the URL.
/// Otherwise, spawns chromedriver on a random port and waits until it accepts connections.
#[allow(clippy::expect_used, clippy::unwrap_used)]
pub fn ensure_chromedriver() -> u16 {
    if let Ok(url) = std::env::var("CHROMEDRIVER_URL") {
        return url
            .rsplit(':')
            .next()
            .and_then(|p| p.parse().ok())
            .unwrap_or(9515);
    }

    let mut guard = match CHROMEDRIVER.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };

    if guard.is_none() {
        let port = portpicker::pick_unused_port().expect("no free port");
        eprintln!("Starting chromedriver on port {port}...");

        let process = spawn_chromedriver(port);

        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while std::time::Instant::now() < deadline {
            if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        eprintln!("Chromedriver ready on port {port}");
        *guard = Some(ChromedriverProcess { process, port });
    }

    guard.as_ref().unwrap().port
}
