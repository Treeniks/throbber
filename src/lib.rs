//! This crate serves as an alternative to [loading](https://crates.io/crates/loading). It is used to display a throbber animation in the terminal while other calculations are done in the main program.
//!
//! # Usage
//! First, add this to your Cargo.toml
//! ```toml
//! [dependencies]
//! throbber = "0.1"
//! ```

use std::io::Write;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// `⠋   ⠙   ⠹   ⠸   ⠼   ⠴   ⠦   ⠧   ⠇   ⠏`
pub const DEFAULT_F: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
/// `◐   ◓   ◑   ◒`
pub const CIRCLE_F: [&str; 4] = ["◐", "◓", "◑", "◒"];
/// `|   /   -   \`
pub const ROTATE_F: [&str; 4] = ["|", "/", "-", "\\"];
/// `[=  ]   [ = ]   [  =]   [ = ]`
pub const MOVE_EQ_F: [&str; 4] = ["[=  ]", "[ = ]", "[  =]", "[ = ]"];
/// `[-  ]   [ - ]   [  -]   [ - ]`
pub const MOVE_MIN_F: [&str; 4] = ["[-  ]", "[ - ]", "[  -]", "[ - ]"];
/// `[=    ]   [==   ]   [ ==  ]   [  == ]   [   ==]   [    =]`
pub const MOVE_EQ_LONG_F: [&str; 10] = [
    "[=    ]", "[==   ]", "[ ==  ]", "[  == ]", "[   ==]", "[    =]", "[   ==]", "[  == ]",
    "[ ==  ]", "[==   ]",
];
/// `[-    ]   [--   ]   [ --  ]   [  -- ]   [   --]   [    -]`
pub const MOVE_MIN_LONG_F: [&str; 10] = [
    "[-    ]", "[--   ]", "[ --  ]", "[  -- ]", "[   --]", "[    -]", "[   --]", "[  -- ]",
    "[ --  ]", "[--   ]",
];

/// Representation of a throbber animation. It can strart, succeed, fail or end at any point.
///
/// Note that a call to `end()` takes ownership of the struct and drops it, as such it should be called to completely remove the throbber animtion object. If you want to start another animation afterwards, you have to create a new `Throbber` object. This is done because multiple calls to start do not actually create multiple threads, instead a call to a *finish function* (like `succeed()`) simply parks the thread and a following `start()` call unparks that thread again. As such, a call to `end()` kills the thread entirely. If you want to just stop the animation, but potentially start it again later on, use `finish()` instead.
///
/// # Examples
/// ```rust
/// use throbber::Throbber;
/// use std::thread;
/// use std::time::Duration;
///
/// let mut throbber = Throbber::new()
///     .message("calculating stuff".to_string())
///     .interval(Duration::from_millis(50))
///     .frames(&throbber::ROTATE_F);
///
/// throbber.start();
///
/// // do stuff
/// thread::sleep(Duration::from_secs(5));
///
/// throbber.success("calculation successful".to_string());
/// throbber.end();
/// ```
pub struct Throbber {
    anim: Option<ThrobberAnim>,
    message: String,
    interval: Duration,
    frames: &'static [&'static str],
}

struct ThrobberAnim {
    thread: JoinHandle<()>,
    sender: Sender<ThrobberSignal>,
}

enum ThrobberSignal {
    Start,
    Finish,
    Succ(String),
    Fail(String),
    ChMsg(String),
    ChInt(Duration),
    ChFrames(&'static [&'static str]),
    End,
}

impl Throbber {
    /// Creates a new Throbber object.
    pub fn new() -> Self {
        Self {
            anim: None,
            message: "".to_string(),
            interval: Duration::from_millis(200),
            frames: &DEFAULT_F,
        }
    }

    /// Sets the message that's supposed to print.
    ///
    /// This does nothing if `start()` was called before. To change the message after `start()` was called, use [`change_message`](Throbber::change_message) instead.
    pub fn message(mut self, msg: String) -> Self {
        self.message = msg;
        self
    }

    /// Sets the interval in which the animation frames are supposed to print.
    ///
    /// This does nothing if `start()` was called before. To change the interval after `start()` was called, use [`change_interval`](Throbber::change_interval) instead.
    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    /// Sets the animation frames that are supposed to print.
    ///
    /// This does nothing if `start()` was called before. To change the animation frames after `start()` was called, use [`change_frames`](Throbber::change_frames) instead.
    pub fn frames(mut self, frames: &'static [&'static str]) -> Self {
        self.frames = frames;
        self
    }

    /// Changes the message that's supposed to print.toml
    ///
    /// Unlike [`message`](Throbber::message), this will work both before and after `start()` was called.
    pub fn change_message(&mut self, msg: String) {
        if let Some(ref anim) = self.anim {
            anim.sender
                .send(ThrobberSignal::ChMsg(msg.clone()))
                .unwrap();
            anim.thread.thread().unpark();
        }
        self.message = msg;
    }

    /// Changes the interval in which the animation frames are supposed to print.
    ///
    /// Unlike [`interval`](Throbber::interval), this will work both before and after `start()` was called.
    pub fn change_interval(&mut self, interval: Duration) {
        if let Some(ref anim) = self.anim {
            anim.sender.send(ThrobberSignal::ChInt(interval)).unwrap();
            anim.thread.thread().unpark();
        }
        self.interval = interval;
    }

    /// Changes the animation frames that are supposed to print.
    ///
    /// Unlike [`frames`](Throbber::frames), this will work both before and after `start()` was called.
    pub fn change_frames(&mut self, frames: &'static [&'static str]) {
        if let Some(ref anim) = self.anim {
            anim.sender.send(ThrobberSignal::ChFrames(frames)).unwrap();
            anim.thread.thread().unpark();
        }
        self.frames = frames;
    }

    /// Starts the animation.
    ///
    /// If this is the first call to start(), a new thread gets created to play the animation. Otherwise the thread that already exists gets unparked and starts the animation again.
    pub fn start(&mut self) {
        if let Some(ref anim) = self.anim {
            anim.sender.send(ThrobberSignal::Start).unwrap();
            anim.thread.thread().unpark();
            return;
        }

        let (sender, receiver): (Sender<ThrobberSignal>, Receiver<ThrobberSignal>) =
            mpsc::channel();

        let msg = self.message.clone();
        let frames = self.frames;
        let interval = self.interval;
        let thread = thread::spawn(move || animation_thread(receiver, msg, frames, interval));

        self.anim = Some(ThrobberAnim { thread, sender });
    }

    /// Starts the animation with the specified `msg`.
    ///
    /// Equivalent to `throbber.change_message(msg); throbber.start();`.
    pub fn start_with_msg(&mut self, msg: String) {
        self.change_message(msg);
        self.start();
    }

    /// Stops the current animation, leaving a blank line.
    pub fn finish(&mut self) {
        if let Some(ref anim) = self.anim {
            anim.sender.send(ThrobberSignal::Finish).unwrap();
            anim.thread.thread().unpark();
        }
    }

    /// Stops the current animation and prints `msg` as a *success message*.
    pub fn success(&mut self, msg: String) {
        if let Some(ref anim) = self.anim {
            anim.sender.send(ThrobberSignal::Succ(msg)).unwrap();
            anim.thread.thread().unpark();
        } else {
            println!("\x1B[2K\r✔ {}", msg);
        }
    }

    /// Stops the current animation and prints `msg` as a *fail message*.
    pub fn fail(&mut self, msg: String) {
        if let Some(ref anim) = self.anim {
            anim.sender.send(ThrobberSignal::Fail(msg)).unwrap();
            anim.thread.thread().unpark();
        } else {
            println!("\x1B[2K\r✖ {}", msg);
        }
    }

    /// Encds the animation thread and drops the throbber object. If you want to stop the animation without dropping the throbber object, use [`finish`](Throbber::finish) instead.
    pub fn end(mut self) {
        if let Some(anim) = self.anim.take() {
            anim.sender.send(ThrobberSignal::End).unwrap();
            anim.thread.thread().unpark();
            anim.thread.join().unwrap();
        }
    }
}

fn animation_thread(
    receiver: Receiver<ThrobberSignal>,
    mut msg: String,
    mut frames: &'static [&'static str],
    mut interval: Duration,
) {
    let mut play_anim = true;
    let mut frame = 0;
    loop {
        match receiver.try_recv() {
            Ok(ThrobberSignal::Start) => {
                play_anim = true;
                continue;
            }
            Ok(ThrobberSignal::Finish) => {
                print!("\x1B[2K\r");
                std::io::stdout().flush().unwrap();
                play_anim = false;
                continue;
            }
            Ok(ThrobberSignal::Succ(succ_msg)) => {
                println!("\x1B[2K\r✔ {}", succ_msg);
                play_anim = false;
                continue;
            }
            Ok(ThrobberSignal::Fail(fail_msg)) => {
                println!("\x1B[2K\r✖ {}", fail_msg);
                play_anim = false;
                continue;
            }
            Ok(ThrobberSignal::ChMsg(new_msg)) => {
                msg = new_msg;
                continue;
            }
            Ok(ThrobberSignal::ChInt(new_dur)) => {
                interval = new_dur;
                continue;
            }
            Ok(ThrobberSignal::ChFrames(new_frames)) => {
                frames = new_frames;
                frame = 0;
                continue;
            }
            Ok(ThrobberSignal::End) => {
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
