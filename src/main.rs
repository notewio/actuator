mod finger;
mod util;

use directories::ProjectDirs;
use evdev::{Device, EventType};
use finger::Finger;
use std::{fs, sync::mpsc};
use toml::Value;
use util::*;

#[derive(Default)]
pub struct Config {
    edge_tolerance: i32,
    min_distance: i32,
    screen_height: i32,
    screen_width: i32,
}

fn main() -> std::io::Result<()> {
    // Read SIGUSR1 to switch screen orientation
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || await_usr1(tx));

    // Load config
    let dir = ProjectDirs::from("", "", "actuator").expect("Could not find configuration folder");
    let dir = dir.config_dir();
    let config_path = dir.join("Actuator.toml");
    let config = fs::read_to_string(config_path)?.parse::<Value>()?;

    let mut input_device = Device::open(config["device"].as_str().unwrap()).unwrap();
    let mut actions = Actions::default();
    if let Value::Table(t) = &config["actions"] {
        for (key, value) in t {
            let v = value
                .as_str()
                .unwrap()
                .split(";")
                .map(|x| x.trim().split(" ").collect())
                .collect::<Vec<Vec<&str>>>();
            if v.len() == 0 {
                continue;
            }
            actions.actions.insert(key.as_str(), v);
        }
    }
    let config = Config {
        edge_tolerance: config["edge_tolerance"].as_integer().unwrap() as i32,
        min_distance: config["min_distance"].as_integer().unwrap() as i32,
        screen_height: config["screen_height"].as_integer().unwrap() as i32,
        screen_width: config["screen_width"].as_integer().unwrap() as i32,
    };

    // State variables
    let mut current_finger = 0usize;
    let mut held_fingers = 0usize;
    let mut fingers = vec![];

    // Main loop
    loop {
        // Switch orientation if there's a signal received
        while rx.try_recv().is_ok() {
            actions.portrait = !actions.portrait;
        }

        // Read device
        for ev in input_device.fetch_events()? {
            if ev.code() == 57 {
                match ev.value() {
                    -1 => held_fingers -= 1,
                    _ => held_fingers += 1,
                }
            } else if ev.code() == 47 {
                current_finger = ev.value() as usize
            }

            if current_finger >= fingers.len() {
                fingers.push(Finger::default());
            }

            if ev.event_type() == EventType::ABSOLUTE {
                match ev.code() {
                    53 => fingers[current_finger].x.push(ev.value()),
                    54 => fingers[current_finger].y.push(ev.value()),
                    _ => {}
                }
            }
        }

        // Is the gesture over, and is it not a tap?
        // NOTE: crazy stupid syntax here. im trying to not have the indents go 1000
        //       miles long with if statements. please breakable blocks when ;_;
        //       RFC 2046
        if held_fingers == 0 && {
            for f in &mut fingers {
                f.delta().unwrap_or_default();
            }
            fingers.iter().any(|x| x.manhattan() > config.min_distance)
        } {
            match fingers.len() {
                1 => actions.run(&gestures_1(&config, &fingers[0]).to_key(1)),
                2 => actions.run(&gestures_2(&config, &fingers[0], &fingers[1]).to_key(2)),
                3 => {
                    // Find the two outside fingers, and run 2finger on them
                    let mut f0 = &fingers[0];
                    let mut f1 = &fingers[1];
                    let f2 = &fingers[2];
                    let d01 = dist2(f0.start(), f1.start());
                    let d02 = dist2(f0.start(), f2.start());
                    let d12 = dist2(f1.start(), f2.start());
                    if d02 >= d01 && d02 >= d12 {
                        f1 = f2;
                    } else if d12 >= d01 && d12 >= d02 {
                        f0 = f2;
                    }
                    actions.run(&gestures_2(&config, f0, f1).to_key(3));
                }
                _ => {}
            };
        }
        // Reset state
        if held_fingers == 0 {
            fingers.clear();
            current_finger = 0;
            held_fingers = 0;
        }
    }
}

fn gestures_1(c: &Config, f: &Finger) -> Direction {
    let (sx, sy) = f.start();

    if f.vertical() {
        if sy < c.edge_tolerance {
            return Direction::FromTop;
        } else if sy > c.screen_height - c.edge_tolerance {
            return Direction::FromBottom;
        } else {
            if f.dy > 0 {
                return Direction::Down;
            } else {
                return Direction::Up;
            }
        }
    } else {
        if sx < c.edge_tolerance {
            return Direction::FromLeft;
        } else if sx > c.screen_width - c.edge_tolerance {
            return Direction::FromRight;
        } else {
            if f.dx > 0 {
                return Direction::Right;
            } else {
                return Direction::Left;
            }
        }
    }
}

fn gestures_2(c: &Config, f0: &Finger, f1: &Finger) -> Direction {
    let (sx0, sy0) = f0.start();
    let (sx1, sy1) = f1.start();

    // ------ Pinch/Spread ------
    // Dot product the two vectors to see if they're parallel
    if f0.dx * f1.dx + f0.dy + f1.dy < 0 {
        let sdist = (sx1 - sx0).pow(2) + (sy1 - sy0).pow(2);
        let (ex0, ey0) = f0.end();
        let (ex1, ey1) = f1.end();
        let edist = (ex1 - ex0).pow(2) + (ey1 - ey0).pow(2);
        if edist < sdist {
            return Direction::Pinch;
        } else {
            return Direction::Spread;
        }
    }
    // ------ Swipe ------
    else {
        // NOTE: finger 0 takes priority
        return gestures_1(c, f0);
    }
}
