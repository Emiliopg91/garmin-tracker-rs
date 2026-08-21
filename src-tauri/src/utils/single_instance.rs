use std::{fs, path::PathBuf, process::Command};

use crate::utils::constants;

pub struct SingleInstance {}

impl SingleInstance {
    pub fn acquire() -> Self {
        if let Ok(content) = fs::read_to_string(&*constants::LOCK_FILE) {
            let pid = content.trim().parse::<u32>().unwrap();
            if let Ok(exists) = fs::exists(PathBuf::from(format!("/proc/{}", pid)))
                && exists
            {
                println!("Stopping previous running instance with PID {}", pid);
                Command::new("kill")
                    .args(["-9", &pid.to_string()])
                    .status()
                    .unwrap();
            }
        }
        let _ = fs::write(&*constants::LOCK_FILE, format!("{}", *constants::PID));

        Self {}
    }
    pub fn release(&self) {
        let _ = fs::remove_file(&*constants::LOCK_FILE);
    }
}
