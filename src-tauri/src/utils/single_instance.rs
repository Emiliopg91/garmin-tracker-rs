use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::io::AsRawFd,
    sync::OnceLock,
    thread,
    time::{Duration, Instant},
};

use crate::utils::constants;

const KILL_GRACE_PERIOD: Duration = Duration::from_secs(3);
const POLL_INTERVAL: Duration = Duration::from_millis(50);

static LOCK_FILE_HANDLE: OnceLock<File> = OnceLock::new();

pub struct SingleInstance {}

impl SingleInstance {
    /// Ensures only one app instance runs: takes an flock on the PID lock file, killing (`SIGTERM` then `SIGKILL`) any previous holder if needed.
    pub fn acquire() {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&*constants::LOCK_FILE)
            .expect("Could not open lock file");

        if !Self::try_lock(&file) {
            let previous_pid = fs::read_to_string(&*constants::LOCK_FILE)
                .ok()
                .and_then(|content| content.trim().parse::<u32>().ok());

            if let Some(pid) = previous_pid {
                println!("Stopping previous running instance with PID {}", pid);
                unsafe { libc::kill(pid as libc::pid_t, libc::SIGTERM) };
            }

            let deadline = Instant::now() + KILL_GRACE_PERIOD;
            let mut acquired = false;
            while Instant::now() < deadline {
                if Self::try_lock(&file) {
                    acquired = true;
                    break;
                }
                thread::sleep(POLL_INTERVAL);
            }

            if !acquired {
                if let Some(pid) = previous_pid {
                    unsafe { libc::kill(pid as libc::pid_t, libc::SIGKILL) };
                }
                Self::lock_blocking(&file);
            }
        }

        let _ = file.set_len(0);
        let _ = (&file).write_all(constants::PID.to_string().as_bytes());

        // Keep the fd open for the process lifetime: dropping it would release the flock.
        let _ = LOCK_FILE_HANDLE.set(file);
    }

    /// Attempts a non-blocking exclusive flock on `file`; returns whether it succeeded.
    fn try_lock(file: &File) -> bool {
        unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) == 0 }
    }

    /// Takes an exclusive flock on `file`, blocking until it's available.
    fn lock_blocking(file: &File) {
        unsafe {
            libc::flock(file.as_raw_fd(), libc::LOCK_EX);
        }
    }
}
