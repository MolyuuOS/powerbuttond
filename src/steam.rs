use std::{error::Error, process::Command};

use log::warn;
use toml::Value;

pub struct Steam {
    username: String,
    home_dir: String,
}

impl Steam {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // Check who is the gamescope login user,
        // read molyuuctl config to get it.
        let molyuuctl_config_content = std::fs::read_to_string("/etc/molyuuctl/config.toml").unwrap();
        let config = molyuuctl_config_content.parse::<Value>().unwrap();
        let user = config["login"]["autologin"]["user"].as_str().unwrap().to_string();
        let mut home_dir = String::new();

        // Find user home directory
        let command = Command::new("getent")
            .arg("passwd")
            .arg(&user)
            .output().unwrap();

        if command.status.success() {
            let output = String::from_utf8_lossy(&command.stdout);
            for line in output.lines() {
                let parts: Vec<&str> = line.split(":").collect();
                if parts[0] == user {
                    home_dir = parts[5].to_string();
                    break;
                }
            }

            if home_dir.is_empty() {
                return Err("Failed to get user home directory".into());
            }
        }

        Ok(Self {
            username: user,
            home_dir,
        })
    }

    pub fn is_gamemode(&self) -> bool {
        let steampid = format!("{}/.steam/steam.pid", self.home_dir);

        if !std::path::Path::new(&steampid).exists() {
            return false;
        }

        let pid_untrim = std::fs::read_to_string(&steampid).unwrap();
        let pid = pid_untrim.trim();
        let cmdline = std::fs::read_to_string(format!("/proc/{}/cmdline", pid)).unwrap();

        cmdline.contains("-gamepadui")
    }

    pub fn run_steam_command(&self, command: &str) {
        let steam_exec = format!("{}/.steam/root/ubuntu12_32/steam", self.home_dir);
        let command = format!("{} -ifrunning {}", steam_exec, command);
        let output = Command::new("su")
            .args([self.username.as_str(), "-c", command.as_str()])
            .output();

        if output.is_err() {
            warn!("Failed to run steam command: {}", command);
            warn!("Error: {}", output.unwrap_err());
        }
    }
}