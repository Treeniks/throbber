//! This crate serves as an alternative to [loading](https://crates.io/crates/loading). It is used to display a loading style animation in the terminal while other calculations are done in the main program.
//!
//! # Usage
//! First, add this to your Cargo.toml
//! ```toml
//! [dependencies]
//! loading_rs = "0.1"
//! ```

use std::io::Write;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub const DEFAULT_F: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
pub const CIRCLE_F: [&str; 4] = ["◐", "◓", "◑", "◒"];
pub const ROTATE_F: [&str; 4] = ["|", "/", "-", "\\"];
pub const MOVE_EQ_F: [&str; 4] = ["[=  ]", "[ = ]", "[  =]", "[ = ]"];
pub const MOVE_MIN_F: [&str; 4] = ["[-  ]", "[ - ]", "[  -]", "[ - ]"];
pub const MOVE_EQ_LONG_F: [&str; 10] = [
    "[=    ]", "[==   ]", "[ ==  ]", "[  == ]", "[   ==]", "[    =]", "[   ==]", "[  == ]",
    "[ ==  ]", "[==   ]",
];
pub const MOVE_MIN_LONG_F: [&str; 10] = [
    "[-    ]", "[--   ]", "[ --  ]", "[  -- ]", "[   --]", "[    -]", "[   --]", "[  -- ]",
    "[ --  ]", "[--   ]",
];

/// Representation of a loading animation. It can strart, succeed, fail or end at any point.
///
/// Note that a call to `end()` takes ownership of the struct and drops it, as such it should be called to completely remove the loading animtion object. If you want to start another animation afterwards, you have to create a new `Loading` object. This is done because multiple calls to start do not actually create multiple threads, instead a call to a *finish function* (like `succeed()`) simply parks the thread and a following `start()` call unparks that thread again. As such, a call to `end()` kills the thread entirely. If you want to just stop the animation, but potentially start it again later on, use `finish()` instead.
pub struct Loading {
    anim: Option<LoadingAnim>,
    message: String,
    interval: Duration,
    frames: &'static [&'static str],
}

struct LoadingAnim {
    thread: JoinHandle<()>,
    sender: Sender<LoadingSignal>,
}

enum LoadingSignal {
    Start,
    Finish,
    Succ(String),
    Fail(String),
    ChMsg(String),
    ChInt(Duration),
    ChFrames(&'static [&'static str]),
    End,
}

impl Loading {
    pub fn new() -> Self {
        Self {
            anim: None,
            message: "".to_string(),
            interval: Duration::from_millis(200),
            frames: &DEFAULT_F,
        }
    }

    pub fn message(mut self, msg: String) -> Self {
        self.message = msg;
        self
    }

    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn frames(mut self, frames: &'static [&'static str]) -> Self {
        self.frames = frames;
        self
    }

    pub fn change_message(&mut self, msg: String) {
        if let Some(ref anim) = self.anim {
            anim.sender.send(LoadingSignal::ChMsg(msg.clone())).unwrap();
            anim.thread.thread().unpark();
        }
        self.message = msg;
    }

    pub fn change_interval(&mut self, interval: Duration) {
        if let Some(ref anim) = self.anim {
            anim.sender.send(LoadingSignal::ChInt(interval)).unwrap();
            anim.thread.thread().unpark();
        }
        self.interval = interval;
    }

    pub fn change_frames(&mut self, frames: &'static [&'static str]) {
        if let Some(ref anim) = self.anim {
            anim.sender.send(LoadingSignal::ChFrames(frames)).unwrap();
            anim.thread.thread().unpark();
        }
        self.frames = frames;
    }

    pub fn start(&mut self) {
        if let Some(ref anim) = self.anim {
            anim.sender.send(LoadingSignal::Start).unwrap();
            anim.thread.thread().unpark();
            return;
        }

        let (sender, receiver): (Sender<LoadingSignal>, Receiver<LoadingSignal>) = mpsc::channel();

        let msg = self.message.clone();
        let frames = self.frames;
        let interval = self.interval;
        let thread = thread::spawn(move || animation_thread(receiver, msg, frames, interval));

        self.anim = Some(LoadingAnim { thread, sender });
    }

    pub fn start_with_msg(&mut self, msg: String) {
        self.change_message(msg);
        self.start();
    }

    pub fn finish(&mut self) {
        if let Some(ref anim) = self.anim {
            anim.sender.send(LoadingSignal::Finish).unwrap();
            anim.thread.thread().unpark();
        }
    }

    pub fn success(&mut self, msg: String) {
        if let Some(ref anim) = self.anim {
            anim.sender.send(LoadingSignal::Succ(msg)).unwrap();
            anim.thread.thread().unpark();
        } else {
            println!("\x1B[2K\r✔ {}", msg);
        }
    }

    pub fn fail(&mut self, msg: String) {
        if let Some(ref anim) = self.anim {
            anim.sender.send(LoadingSignal::Fail(msg)).unwrap();
            anim.thread.thread().unpark();
        } else {
            println!("\x1B[2K\r✖ {}", msg);
        }
    }

    pub fn end(mut self) {
        if let Some(anim) = self.anim.take() {
            anim.sender.send(LoadingSignal::End).unwrap();
            anim.thread.thread().unpark();
            anim.thread.join().unwrap();
        }
    }
}

fn animation_thread(
    receiver: Receiver<LoadingSignal>,
    mut msg: String,
    mut frames: &'static [&'static str],
    mut interval: Duration,
) {
    let mut play_anim = true;
    let mut frame = 0;
    loop {
        match receiver.try_recv() {
            Ok(LoadingSignal::Start) => {
                play_anim = true;
                continue;
            }
            Ok(LoadingSignal::Finish) => {
                print!("\x1B[2K\r");
                std::io::stdout().flush().unwrap();
                play_anim = false;
                continue;
            }
            Ok(LoadingSignal::Succ(succ_msg)) => {
                println!("\x1B[2K\r✔ {}", succ_msg);
                play_anim = false;
                continue;
            }
            Ok(LoadingSignal::Fail(fail_msg)) => {
                println!("\x1B[2K\r✖ {}", fail_msg);
                play_anim = false;
                continue;
            }
            Ok(LoadingSignal::ChMsg(new_msg)) => {
                msg = new_msg;
                continue;
            }
            Ok(LoadingSignal::ChInt(new_dur)) => {
                interval = new_dur;
                continue;
            }
            Ok(LoadingSignal::ChFrames(new_frames)) => {
                frames = new_frames;
                frame = 0;
                continue;
            }
            Ok(LoadingSignal::End) => {
                break;
            }
            Err(TryRecvError::Disconnected) => {
                break;
            }
            Err(TryRecvError::Empty) => {
                if play_anim == false {
                    thread::park();
                    continue;
                }
            }
        }
        print!("\x1B[2K\r");
        print!("{} {}", frames[frame], msg);
        std::io::stdout().flush().unwrap();
        thread::sleep(interval);
        frame = (frame + 1) % frames.len();
    }
}
