use std::{process::{Command, exit}, time::Duration};

use log::{error, info};

use powerbutton::PowerButton;
use steam::Steam;

use crate::errors::system::LockError;

mod errors;
mod lock;
mod logger;
mod steam;
mod powerbutton;

static STEAMDECK_POWERKEY: &'static str = "/dev/input/by-path/platform-i8042-serio-0-event-kbd";
static POWERBUTTOND_LOCKNAME: &'static str = "molyuuos-powerbuttond";
static LONG_PRESS_THRESHOLD: Duration = Duration::from_millis(1000);

#[tokio::main]
async fn main() {
    logger::init().unwrap();

    let mut lock = lock::Lock::new(POWERBUTTOND_LOCKNAME, None);
    match lock.lock() {
        Ok(_) => {
            info!("MolyuuOS Power Button Daemon Started!");
        }
        Err(err) => {
            error!("MolyuuOS Power Button Daemon Failed to Start: {}", err);
            // Gamescope may start the daemon, silently exit if the lock is already held
            if *err.downcast_ref::<LockError>().unwrap() == LockError::FileIsLocked {
                exit(0)
            } else {
                exit(1)
            }
        }
    }

    let steam = Steam::new().unwrap();
    let power_button = PowerButton::new(STEAMDECK_POWERKEY, LONG_PRESS_THRESHOLD, |press_type| {
        let is_gamemode = steam.is_gamemode();
        match press_type {
            powerbutton::PressType::ShortPress => {
                if is_gamemode {
                    steam.run_steam_command("steam://shortpowerpress");
                } else {
                    Command::new("systemctl")
                        .arg("suspend")
                        .output().unwrap();
                }
            }
            powerbutton::PressType::LongPress => {
                if is_gamemode {
                    steam.run_steam_command("steam://longpowerpress");
                } else {
                    Command::new("systemctl")
                        .arg("poweroff")
                        .output().unwrap();
                }
            }
        }
    });

    power_button.handle().await;
}
