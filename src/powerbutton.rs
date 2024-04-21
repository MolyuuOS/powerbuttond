use std::time::{Duration, Instant};

use evdev::{Device, InputEventKind, Key};
use log::info;

pub enum PressType {
    LongPress,
    ShortPress,
}

pub struct PowerButton<'a> {
    device_path: String,
    long_press_threshold: Duration,
    on_pressed: Box<dyn Fn(PressType) + 'a>,
}

impl<'a> PowerButton<'a> {
    pub fn new<F>(path: &str, long_press_threshold: Duration, on_pressed: F) -> Self
        where F: Fn(PressType) + 'a {
        Self {
            device_path: path.to_string(),
            long_press_threshold,
            on_pressed: Box::new(on_pressed),
        }
    }

    pub async fn handle(&self) {
        let device = Device::open(self.device_path.as_str()).unwrap();
        let mut pressed_time = Instant::now();
        let mut stream = device.into_event_stream().unwrap();
        loop {
            match stream.next_event().await {
                Ok(event) => {
                    if event.kind() == InputEventKind::Key(Key::KEY_POWER) {
                        if event.value() == 1 {
                            pressed_time = Instant::now();
                        } else if pressed_time.elapsed() < self.long_press_threshold {
                            (self.on_pressed)(PressType::ShortPress);
                            info!("Power button short pressed!");
                        } else {
                            (self.on_pressed)(PressType::LongPress);
                            info!("Power button long pressed!");
                        }
                    }
                }
                Err(err) => {
                    log::error!("Failed to read event: {}", err);
                    break;
                }
            }
        }
    }
}