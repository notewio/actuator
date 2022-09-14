use directories::ProjectDirs;
use evdev::Device;
use signal_hook::consts::SIGUSR1;
use signal_hook::iterator::Signals;
use std::fs;
use std::process::Command;
use std::sync::mpsc;
use std::thread;

type BoxResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Default, Debug)]
struct Finger {
    x: Vec<i32>,
    y: Vec<i32>,
}
impl Finger {
    fn push_x(&mut self, value: i32) {
        if self.x.len() < 2 {
            self.x.push(value);
        } else {
            self.x[1] = value;
        }
    }
    fn push_y(&mut self, value: i32) {
        if self.y.len() < 2 {
            self.y.push(value);
        } else {
            self.y[1] = value;
        }
    }
    fn dx(&self) -> i32 {
        return self.x.last().unwrap_or(&0) - self.x.get(0).unwrap_or(&0);
    }
    fn dy(&self) -> i32 {
        return self.y.last().unwrap_or(&0) - self.y.get(0).unwrap_or(&0);
    }
    fn dist2(&self) -> i32 {
        return self.dx().pow(2) + self.dy().pow(2);
    }
    fn vertical(&self) -> bool {
        return self.dy().abs() > self.dx().abs();
    }
    fn start(&self) -> (i32, i32) {
        return (self.x[0], self.y[0]);
    }
}

fn main() -> BoxResult<()> {
    let dir = ProjectDirs::from("", "", "actuator").expect("Could not find configuration folder");
    let path = dir.config_dir().join("actuator.toml");
    let config: toml::Value = toml::from_str(&fs::read_to_string(path)?)?;

    let mut device = Device::open(value_get_string(&config, "device"))?;

    let min_distance2 = value_get_int(&config, "min_distance");
    let dimensions = (
        value_get_int(&config, "width"),
        value_get_int(&config, "height"),
        value_get_int(&config, "edge_tolerance"),
    );
    let actions = config
        .get("actions")
        .expect("Missing [actions] section in config");

    let mut fingers: Vec<Finger> = vec![];
    let mut slot = 0usize;
    let mut held = 0usize;
    let mut portrait = false;

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut signals = Signals::new(&[SIGUSR1]).expect("Failed to open signal hook");
        for _ in &mut signals {
            match tx.send(()) {
                Ok(()) => {}
                Err(e) => println!("Error sending signal across threads: {e}"),
            };
        }
    });

    loop {
        while let Ok(_) = rx.try_recv() {
            portrait = !portrait;
        }

        for ev in device.fetch_events()? {
            match ev.code() {
                47 => slot = ev.value() as usize,
                53 => fingers[slot].push_x(ev.value()),
                54 => fingers[slot].push_y(ev.value()),
                57 => match ev.value() {
                    -1 => held -= 1,
                    _ => held += 1,
                },
                _ => {}
            }
            if slot >= fingers.len() {
                fingers.push(Finger::default());
            }
        }

        if held == 0 {
            if fingers.iter().any(|x| x.dist2() > min_distance2) {
                let action = format!(
                    "{}_{}",
                    fingers.len(),
                    gestures(dimensions, &fingers[0], portrait)
                );
                match actions.get(&action) {
                    Some(toml::Value::String(value)) => {
                        let lines = value.split(";");
                        for command in lines {
                            let parts: Vec<&str> = command.split(" ").collect();
                            match Command::new(parts[0]).args(&parts[1..]).spawn() {
                                Err(e) => eprintln!("Action {action} could not be run: {e}"),
                                _ => {}
                            }
                        }
                    }
                    Some(_) => eprintln!("Action {action} is not a string"),
                    _ => {}
                }
            }

            fingers.clear();
            slot = 0;
        }
    }
}

fn gestures(dimensions: (i32, i32, i32), f: &Finger, portrait: bool) -> &str {
    let (mut sx, mut sy) = f.start();
    let (mut sw, mut sh, tolerance) = dimensions;

    if portrait {
        std::mem::swap(&mut sx, &mut sy);
        std::mem::swap(&mut sw, &mut sh);
    }

    if f.vertical() ^ portrait {
        if sy < tolerance {
            return "from_top";
        } else if sy > sh - tolerance {
            return "from_bottom";
        } else {
            if f.dy() > 0 {
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
            if f.dx() > 0 {
                return "right";
            } else {
                return "left";
            }
        }
    }
}

fn value_get_int(config: &toml::Value, key: &str) -> i32 {
    return config
        .get(key)
        .expect(&format!("Missing config field: {key}"))
        .as_integer()
        .expect(&format!("{key} is not an integer")) as i32;
}
fn value_get_string(config: &toml::Value, key: &str) -> String {
    return config
        .get(key)
        .expect(&format!("Missing config field: {key}"))
        .as_str()
        .expect(&format!("{key} is not a string"))
        .to_string();
}
