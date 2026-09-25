pub mod errors;

use std::{
    io::Write,
    os::unix::fs::PermissionsExt,
    process::{Command, ExitStatus, Stdio},
};

use errors::{Result, UdevError};

use crate::utils::constants;

pub struct UdevManager {}

impl UdevManager {
    pub fn is_root() -> bool {
        unsafe { libc::geteuid() == 0 }
    }

    pub fn write_rules_file(auto_run: bool) -> Result<()> {
        if Self::is_root() {
            let mut content =
                include_str!("../../../resources/99-garmin-tracker-rs.rules").to_string();
            if auto_run {
                content.push('\n');
                content.push_str(include_str!(
                    "../../../resources/99-garmin-tracker-rs-launch.rules"
                ));
            }
            std::fs::write(constants::RULE_FILE, content).map_err(UdevError::Write)?;

            UdevManager::reload()?;
            UdevManager::trigger()?;
        } else {
            let current_exe = std::env::current_exe().unwrap();

            let mut askpass_file = tempfile::Builder::new()
                .prefix(&format!("{}-askpass-", *constants::APP_NAME))
                .tempfile()
                .map_err(UdevError::Write)?;

            let askpass_content = r#"#!/bin/bash
zenity --password --title="Elevated permissions are required to edit device rules"
"#;
            askpass_file
                .write_all(askpass_content.as_bytes())
                .map_err(UdevError::Write)?;
            askpass_file
                .as_file()
                .set_permissions(std::fs::Permissions::from_mode(0o700))
                .map_err(UdevError::Write)?;

            let askpass_path = askpass_file.into_temp_path();

            let status = Command::new("sudo")
                .arg("-k")
                .arg("-A")
                .arg(&current_exe)
                .arg("--rules")
                .arg(auto_run.to_string())
                .env("SUDO_ASKPASS", &askpass_path)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .map_err(UdevError::Reload)?;

            Self::handle_command_status(&status)?;
        }

        Ok(())
    }

    pub fn reload() -> Result<()> {
        eprintln!("Reloading udev rules...");
        let status = Command::new("udevadm")
            .arg("control")
            .arg("--reload-rules")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(UdevError::Reload)?;

        Self::handle_command_status(&status)
    }
    pub fn trigger() -> Result<()> {
        eprintln!("Triggering udev rules...");
        let status = Command::new("udevadm")
            .arg("trigger")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(UdevError::Trigger)?;

        Self::handle_command_status(&status)
    }

    fn handle_command_status(status: &ExitStatus) -> Result<()> {
        if status.success() {
            Ok(())
        } else {
            let code = status.code();
            Err(UdevError::Reload(std::io::Error::other(format!(
                "Bad exit status: {:?}",
                code
            ))))
        }
    }
}
