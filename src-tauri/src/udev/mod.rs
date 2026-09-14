pub mod errors;

use std::{
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
            let args = std::env::args().collect::<Vec<String>>();
            let arg0 = args.first().unwrap();

            let askpass_file = format!("/tmp/{}-askpass", *constants::APP_NAME);
            if !std::fs::exists(&askpass_file).unwrap() {
                let askpass_content = r#"#!/bin/bash
zenity --password --title="Elevated permissions are required to edit device rules"
"#;
                let _ = std::fs::write(&askpass_file, askpass_content);
                let permisos = std::fs::Permissions::from_mode(0o555);
                let _ = std::fs::set_permissions(&askpass_file, permisos);
            }

            let status = Command::new("sudo")
                .arg("-k")
                .arg("-A")
                .arg(arg0)
                .arg("--rules")
                .arg(auto_run.to_string())
                .env("SUDO_ASKPASS", askpass_file)
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
