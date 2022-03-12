use signal_hook::{consts::SIGUSR1, iterator::Signals};
use std::{collections::HashMap, process::Command, sync::mpsc};

pub fn await_usr1(signal_channel: mpsc::Sender<()>) {
    let mut signals = Signals::new(&[SIGUSR1]).unwrap();

    for _ in &mut signals {
        signal_channel.send(()).unwrap();
    }
}

pub enum Direction {
    FromTop,
    FromBottom,
    FromLeft,
    FromRight,
    Left,
    Right,
    Up,
    Down,
    Pinch,
    Spread,
}

impl Direction {
    pub fn to_key(&self, n: usize) -> String {
        n.to_string()
            + match self {
                Direction::FromTop => "_from_top",
                Direction::FromBottom => "_from_bottom",
                Direction::FromLeft => "_from_left",
                Direction::FromRight => "_from_right",
                Direction::Left => "_left",
                Direction::Right => "_right",
                Direction::Up => "_up",
                Direction::Down => "_down",
                Direction::Pinch => "_pinch",
                Direction::Spread => "_spread",
            }
    }
}

pub fn dist2(a: (i32, i32), b: (i32, i32)) -> i32 {
    (b.0 - a.0).pow(2) + (b.1 - a.1).pow(2)
}

#[derive(Default)]
pub struct Actions<'a> {
    pub portrait: bool,
    pub actions: HashMap<&'a str, Vec<Vec<&'a str>>>,
}

impl Actions<'_> {
    pub fn run(&self, a: &str) {
        println!("{}", a);
        let v = self.actions.get(a);
        if let Some(v) = v {
            for c in v {
                Command::new(c[0])
                    .args(&c[1..])
                    .spawn()
                    .expect("Failed to run action");
            }
        }
    }
}
