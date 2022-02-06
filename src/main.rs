mod finger;
#[macro_use]
mod util;

use directories::BaseDirs;
use evdev::{Device, EventType};
use finger::Finger;
use std::{collections::HashMap, fs, process::Command, sync::mpsc};
use toml::Value;
use util::*;

fn main() {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || await_usr1(tx));

    let base_dirs = BaseDirs::new().expect("Could not find configuration file folder");
    let config_path = base_dirs
        .config_dir()
        .join("actuator/Actuator.toml")
        .into_os_string()
        .into_string()
        .unwrap();
    let config = fs::read_to_string(config_path)
        .expect("Could not read config file")
        .parse::<Value>()
        .unwrap();
    let mut input_device = Device::open(config["device"].as_str().unwrap()).unwrap();
    let edge_tolerance = config["edge_tolerance"].as_integer().unwrap() as i32;
    let min_distance = config["min_distance"].as_integer().unwrap() as i32;
    let screen_height = config["screen_height"].as_integer().unwrap() as i32;
    let screen_width = config["screen_width"].as_integer().unwrap() as i32;
    let mut actions = HashMap::new();
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
            actions.insert(key.as_str(), v);
        }
    }

    let mut current_finger = 0;
    let mut held_fingers = 0;
    let mut fingers = vec![];
    let mut portrait = false;
    loop {
        while let Ok(_) = rx.try_recv() {
            portrait = !portrait;
        }
        for ev in input_device.fetch_events().unwrap() {
            if ev.code() == 57 && ev.value() == -1 {
                held_fingers -= 1;
            } else if ev.code() == 57 && ev.value() >= 0 {
                held_fingers += 1;
            } else if ev.code() == 47 {
                current_finger = ev.value() as usize;
            }

            if current_finger >= fingers.len() {
                fingers.push(Finger::new());
            }

            if ev.event_type() == EventType::ABSOLUTE {
                match ev.code() {
                    53 => fingers[current_finger].x.push(ev.value()),
                    54 => fingers[current_finger].y.push(ev.value()),
                    _ => {}
                }
            }
        }

        if held_fingers == 0 {
            match fingers.len() {
                1 => {
                    let f = &fingers[0];
                    if !f.empty() {
                        let (sx, sy) = f.start();
                        let (dx, dy) = f.delta();
                        if dx.abs() + dy.abs() < min_distance {
                        } else if dy.abs() > dx.abs() {
                            if sy < edge_tolerance {
                                run_action!(portrait, actions, "1_from_up");
                            } else if sy > screen_height - edge_tolerance {
                                run_action!(portrait, actions, "1_from_down");
                            }
                        } else {
                            if sx < edge_tolerance {
                                run_action!(portrait, actions, "1_from_left");
                            }
                            if sx > screen_width - edge_tolerance {
                                run_action!(portrait, actions, "1_from_right");
                            }
                        }
                    }
                }
                2 => {
                    let f0 = &fingers[0];
                    let f1 = &fingers[1];
                    if !(f0.empty() || f1.empty()) {
                        let (dx0, dy0) = f0.delta();
                        let (dx1, dy1) = f1.delta();

                        if dx0.abs() + dy0.abs() < min_distance
                            || dx1.abs() + dy1.abs() < min_distance
                        {
                        } else if dx0 * dx1 + dy0 * dy1 < 0 {
                            let (sx0, sy0) = f0.start();
                            let (sx1, sy1) = f1.start();
                            let sdist = (sx1 - sx0).pow(2) + (sy1 - sy0).pow(2);
                            let (ex0, ey0) = f0.end();
                            let (ex1, ey1) = f1.end();
                            let edist = (ex1 - ex0).pow(2) + (ey1 - ey0).pow(2);
                            if edist < sdist {
                                run_action!(portrait, actions, "2_pinch");
                            } else {
                                run_action!(portrait, actions, "2_spread");
                            }
                        } else {
                            // finger 0 takes priority. is it worth calculating average here?...
                            if dy0.abs() > dx0.abs() {
                                if dy0 > 0 {
                                    run_action!(portrait, actions, "2_down");
                                } else {
                                    run_action!(portrait, actions, "2_up");
                                }
                            } else {
                                if dx0 > 0 {
                                    run_action!(portrait, actions, "2_right");
                                } else {
                                    run_action!(portrait, actions, "2_left");
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            fingers.clear();
            held_fingers = 0;
            current_finger = 0;
        }
    }
}
