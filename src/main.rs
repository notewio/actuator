mod finger;

use directories::ProjectDirs;
use evdev::Device;
use finger::Finger;
use serde::Deserialize;
use signal_hook::consts::SIGUSR1;
use signal_hook::iterator::Signals;
use std::env;
use std::fs;
use std::process::Command;
use std::sync::mpsc;
use std::thread;

type BoxResult<T> = Result<T, Box<dyn std::error::Error>>;

const EVENT_ABS_X: u16 = 0x35;
const EVENT_ABS_Y: u16 = 0x36;
const EVENT_ID: u16 = 0x39;
const EVENT_SLOT: u16 = 0x2f;

#[derive(Deserialize)]
struct Config {
    device: String,
    actions: toml::Value,

    width: i32,
    height: i32,
    edge_tolerance: i32,
    min_distance: i32,
}
#[derive(Default, Debug)]
struct State {
    fingers: Vec<Finger>,
    slot: usize,
    current: usize,
    held: usize,
    portrait: bool,
}

fn main() -> BoxResult<()> {
    // Load config
    let config: Config = {
        let path = ProjectDirs::from("", "", "actuator")
            .ok_or("No config folder")?
            .config_dir()
            .join("actuator.toml");
        toml::from_str(&fs::read_to_string(path)?)?
    };
    let mut device = Device::open(&config.device)?;
    let mut state = State::default();
    let shell = env::var("SHELL").expect("No $SHELL environment variable");

    // Start SIGUSR1 listening thread
    let (tx, signals) = mpsc::channel();
    thread::spawn(move || {
        let mut signals = Signals::new(&[SIGUSR1]).expect("Failed to open signal hook");
        for _ in &mut signals {
            match tx.send(()) {
                _ => {}
            };
        }
    });

    // Main loop
    loop {
        // Check for portrait signals
        while let Ok(()) = signals.try_recv() {
            state.portrait = !state.portrait;
        }

        // Read device events
        for ev in device.fetch_events()? {
            match ev.code() {
                EVENT_ID => match ev.value() {
                    -1 => state.current -= 1,
                    _ => {
                        state.current += 1;
                        state.held += 1;
                    }
                },
                EVENT_SLOT => state.slot = ev.value() as usize,
                EVENT_ABS_X => state.fingers[state.slot].push_x(ev.value()),
                EVENT_ABS_Y => state.fingers[state.slot].push_y(ev.value()),
                _ => {}
            }
            if state.slot >= state.fingers.len() {
                state.fingers.push(Finger::default());
            }
        }

        // Run gesture
        if state.current == 0 {
            // If any finger has moved far enough
            if state
                .fingers
                .iter()
                .map(Finger::delta)
                .any(|d| dist2(d) > config.min_distance.pow(2))
            {
                let action = format!(
                    "{}_{}",
                    state.held,
                    gesture(&config, &state.fingers[0], state.portrait)
                );

                match config.actions.get(&action) {
                    Some(toml::Value::String(value)) => {
                        let c = Command::new(&shell).arg("-c").arg(value).spawn();
                        thread::spawn(move || {
                            if let Ok(mut child) = c {
                                child.wait().expect("Action {action} not running");
                            } else {
                                eprintln!("Action {action} did not start");
                            }
                        });
                    }
                    Some(_) => eprintln!("Action {action} is not a string"),
                    _ => {}
                }
            }

            for finger in &mut state.fingers {
                finger.clear();
            }
            state.slot = 0;
            state.held = 0;
        }
    }
}

fn gesture(config: &Config, f: &Finger, portrait: bool) -> &'static str {
    let (mut sx, mut sy) = f.start();
    let (mut sw, mut sh) = (config.width, config.height);
    let tolerance = config.edge_tolerance;

    if portrait {
        std::mem::swap(&mut sx, &mut sy);
        std::mem::swap(&mut sw, &mut sh);
    }

    let (dx, dy) = f.delta();

    if (dy.abs() > dx.abs()) ^ portrait {
        if sy < tolerance {
            return "from_top";
        } else if sy > sh - tolerance {
            return "from_bottom";
        } else {
            if dy > 0 {
                return "down";
            } else {
                return "up";
            }
        }
    } else {
        if sx < tolerance {
            return "from_left";
        } else if sx > sw - tolerance {
            return "from_right";
        } else {
            if dx > 0 {
                return "right";
            } else {
                return "left";
            }
        }
    }
}

fn dist2(delta: (i32, i32)) -> i32 {
    delta.0.pow(2) + delta.1.pow(2)
}
