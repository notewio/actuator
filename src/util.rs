use signal_hook::consts::*;
use signal_hook::iterator::Signals;
use std::sync::mpsc;

pub fn await_usr1(signal_channel: mpsc::Sender<()>) {
    let mut signals = Signals::new(&[SIGUSR1]).unwrap();

    for _ in &mut signals {
        signal_channel.send(()).unwrap();
    }
}

#[macro_export]
macro_rules! run_action {
    ($p: expr, $a: expr, $k: expr) => {
        let mut action = $k.to_string();
        if $p {
            if action.contains("up") {
                action = action.replace("up", "right");
            } else if action.contains("down") {
                action = action.replace("down", "left");
            } else if action.contains("left") {
                action = action.replace("left", "down");
            } else if action.contains("right") {
                action = action.replace("right", "up");
            }
        }
        println!("{}", action);
        let a = $a.get(action.as_str());
        if let Some(v) = a {
            for c in v {
                Command::new(c[0])
                    .args(&c[1..])
                    .spawn()
                    .expect("Failed to run action");
            }
        } else {
            println!("No action found");
        }
    };
}
