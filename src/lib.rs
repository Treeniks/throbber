//! [![Crates.io](https://img.shields.io/crates/v/throbber)](https://crates.io/crates/throbber)
//! [![docs.rs](https://docs.rs/throbber/badge.svg)](https://docs.rs/throbber)
//! [![GitHub last commit](https://img.shields.io/github/last-commit/Treeniks/throbber)](https://github.com/Treeniks/throbber)
//! [![License](https://img.shields.io/github/license/Treeniks/throbber)](https://github.com/Treeniks/throbber/blob/master/LICENSE)
//!
//! This crate serves as an alternative to [loading](https://crates.io/crates/loading). It is used to display a throbber animation in the terminal while other calculations are done in the main program.
//!
//! ![Throbber Preview](https://user-images.githubusercontent.com/56131826/109326392-68c28b00-7857-11eb-8e8d-dd576c868e7f.gif "Throbber Preview")
//!
//! # Usage
//!
//! Add this to your Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! throbber = "0.1"
//! ```
//!
//! To display a throbber animation, first create a [`Throbber`](Throbber) object:
//!
//! ```rust
//! # use throbber::Throbber;
//! let mut throbber = Throbber::new();
//! ```
//!
//! You can also customize certain settings like the displayed animation and the displayed message:
//!
//! ```rust
//! # use throbber::Throbber;
//! let mut throbber = Throbber::new()
//!     .message("calculating stuff".to_string())
//!     .frames(&throbber::MOVE_EQ_F); // this crate comes with a few predefined animations
//!                                    // see the Constants section
//! ```
//!
//! Then you can simply call [`start`](Throbber::start) wherever you want to start the animation and a *finish function* like [`success`](Throbber::success) where you want to stop it.
//!
//! ```rust
//! # use throbber::Throbber;
//! # let mut throbber = Throbber::new();
//! throbber.start();
//! // do calculations
//! throbber.success("calculations successful!".to_string());
//! ```
//!
//! After which you can call [`start`](Throbber::start) or [`start_with_msg`](Throbber::start_with_msg) again to start the animation again.
//! You can also change everything you could customize during the Throbber object creation, e. g. with [`change_message`](Throbber::change_message) and [`change_frames`](Throbber::change_frames). This also works while an animation is running.
//!
//! If you don't intend to start another animation, you should drop the Throbber object with [`end`](Throbber::end). This action also ends the underlying thread:
//!
//! ```rust
//! # use throbber::Throbber;
//! # let mut throbber = Throbber::new();
//! throbber.end();
//! ```
//!
//! # Examples
//!
//! This is the example from the preview above:
//!
//! ```rust
//! use std::thread;
//! use std::time::Duration;
//! use throbber::Throbber;
//!
//! fn main() {
//!     let mut throbber = Throbber::new().message("calculating stuff".to_string());
//!
//!     throbber.start();
//!     // do stuff
//!     thread::sleep(Duration::from_secs(2));
//!     throbber.success("Success".to_string());
//!
//!     throbber.start_with_msg("calculating more stuff".to_string());
//!     // do other stuff
//!     thread::sleep(Duration::from_secs(2));
//!     throbber.fail("Fail".to_string());
//!
//!     throbber.end();
//! }
//! ```
//!
//! You can also keep track of progress with [`change_message`](Throbber::change_message):
//!
//! ```rust
//! use std::thread;
//! use std::time::Duration;
//! use throbber::Throbber;
//!
//! fn main() {
//!     let mut throbber = Throbber::new()
//!         .message("Downloading file1 0%".to_string())
//!         .frames(&throbber::ROTATE_F)
//!         .interval(Duration::from_millis(100));
//!
//!     throbber.start();
//!     for i in 0..100 {
//!         throbber.change_message(format!("Downloading file1 {}%", i));
//!         thread::sleep(Duration::from_millis(30));
//!     }
//!     throbber.success("Downloaded file1".to_string());
//!
//!     throbber.start_with_msg("Downloading file2 0%".to_string());
//!     for i in 0..69 {
//!         throbber.change_message(format!("Downloading file2 {}%", i));
//!         thread::sleep(Duration::from_millis(30));
//!     }
//!     throbber.fail("Download of file2 failed".to_string());
//!
//!     throbber.end();
//! }
//! ```

use std::io::Write;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// `⠋   ⠙   ⠹   ⠸   ⠼   ⠴   ⠦   ⠧   ⠇   ⠏`
///
/// This is the default animation when creating a new Throbber object.
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

/// Representation of a throbber animation. It can start, succeed, fail or end at any point.
///
/// Note that a call to [`end`](Throbber::end) takes ownership of the struct and drops it, as such it should be called to completely remove the throbber animtion object. If you want to start another animation afterwards, you have to create a new [`Throbber`](Throbber) object. This is done because multiple calls to start do not actually create multiple threads, instead a call to a *finish function* (like [`success`](Throbber::success)) simply parks the thread and a following [`start`](Throbber::start) call unparks that thread again. As such, a call to [`end`](Throbber::end) kills the thread entirely. If you want to just stop the animation, but potentially start it again later on, use [`finish`](Throbber::finish) instead.
///
/// # Examples
///
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
    ///
    /// # Default Values
    ///
    /// If you do not customize your throbber animation with [`message`](Throbber::message) etc., these are the default values:
    ///
    /// * message: `""`
    /// * interval: `Duration::from_millis(200)`
    /// * frames: `DEFAULT_F (⠋   ⠙   ⠹   ⠸   ⠼   ⠴   ⠦   ⠧   ⠇   ⠏)`
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
    /// This does nothing if [`start`](Throbber::start) was called before. To change the message after [`start`](Throbber::start) was called, use [`change_message`](Throbber::change_message) instead.
    pub fn message(mut self, msg: String) -> Self {
        self.message = msg;
        self
    }

    /// Sets the interval in which the animation frames are supposed to print.
    ///
    /// This does nothing if [`start`](Throbber::start) was called before. To change the interval after [`start`](Throbber::start) was called, use [`change_interval`](Throbber::change_interval) instead.
    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    /// Sets the animation frames that are supposed to print.
    ///
    /// This does nothing if [`start`](Throbber::start) was called before. To change the animation frames after [`start`](Throbber::start) was called, use [`change_frames`](Throbber::change_frames) instead.
    pub fn frames(mut self, frames: &'static [&'static str]) -> Self {
        self.frames = frames;
        self
    }

    /// Changes the message that's supposed to print.toml
    ///
    /// Unlike [`message`](Throbber::message), this will work both before and after [`start`](Throbber::start) was called.
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
    /// Unlike [`interval`](Throbber::interval), this will work both before and after [`start`](Throbber::start) was called.
    pub fn change_interval(&mut self, interval: Duration) {
        if let Some(ref anim) = self.anim {
            anim.sender.send(ThrobberSignal::ChInt(interval)).unwrap();
            anim.thread.thread().unpark();
        }
        self.interval = interval;
    }

    /// Changes the animation frames that are supposed to print.
    ///
    /// Unlike [`frames`](Throbber::frames), this will work both before and after [`start`](Throbber::start) was called.
    pub fn change_frames(&mut self, frames: &'static [&'static str]) {
        if let Some(ref anim) = self.anim {
            anim.sender.send(ThrobberSignal::ChFrames(frames)).unwrap();
            anim.thread.thread().unpark();
        }
        self.frames = frames;
    }

    /// Starts the animation.
    ///
    /// If this is the first call to [`start`](Throbber::start), a new thread gets created to play the animation. Otherwise the thread that already exists gets unparked and starts the animation again.
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
    ///
    /// This does currently **not** print the fail message onto stderr, but stdout instead. That might change in a future version.
    pub fn fail(&mut self, msg: String) {
        if let Some(ref anim) = self.anim {
            anim.sender.send(ThrobberSignal::Fail(msg)).unwrap();
            anim.thread.thread().unpark();
        } else {
            println!("\x1B[2K\r✖ {}", msg);
        }
    }

    /// Ends the animation thread and drops the throbber object. If you want to stop the animation without dropping the throbber object, use [`finish`](Throbber::finish) instead.
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
                print!("\x1B[2K\r");
                std::io::stdout().flush().unwrap();
                break;
            }
            Err(TryRecvError::Disconnected) => {
                print!("\x1B[2K\r");
                std::io::stdout().flush().unwrap();
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
