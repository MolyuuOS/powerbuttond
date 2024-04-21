use std::{process::Command, time::Duration};

use log::info;

use powerbutton::PowerButton;
use steam::Steam;

mod logger;
mod steam;
mod powerbutton;

static STEAMDECK_POWERKEY: &'static str = "/dev/input/by-path/platform-i8042-serio-0-event-kbd";
static LONG_PRESS_THRESHOLD: Duration = Duration::from_millis(1000);

#[tokio::main]
async fn main() {
    logger::init().unwrap();

    info!("MolyuuOS Power Button Daemon Started!");
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
