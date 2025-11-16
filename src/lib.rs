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
//! throbber = "1.0"
//! ```
//!
//! To display a throbber animation, first create a [`Throbber`] object:
//!
//! ```rust
//! use throbber::Throbber;
//! let mut throbber = Throbber::default();
//! ```
//!
//! You can also customize certain settings like the displayed animation and the displayed message:
//!
//! ```rust
//! # use throbber::Throbber;
//! let mut throbber = Throbber::default()
//!     .message("calculating stuff")
//!     .frames(&throbber::MOVE_EQ_F); // this crate comes with a few predefined animations
//!                                    // see the Constants section
//! ```
//!
//! Then you can simply call [`start`] wherever you want to start the animation and a _finish function_ like [`success`] where you want to stop it.
//!
//! ```rust
//! # use throbber::Throbber;
//! # let mut throbber = Throbber::default();
//! throbber.start();
//! // do calculations
//! throbber.success("calculations successful!");
//! ```
//!
//! After, you can call [`start`] or [`start_with_msg`] again to start the animation again.
//! Setters are also provided, e.g. [`set_message`] and [`set_frames`]. This also works while an animation is running.
//!
//! ## Thread Lifetime
//!
//! The Throbber thread gets spawned on the first call to [`start`] or [`start_with_msg`]. After that, the thread only ever gets parked.
//! If you want to end the thread, you must drop the Throbber object:
//!
//! ```rust
//! # use throbber::Throbber;
//! # let mut throbber = Throbber::default();
//! drop(throbber);
//! ```
//!
//! # Examples
//!
//! This is the example from the gif above:
//!
//! ```rust
#![doc = include_str!("../examples/calculation.rs")]
//! ```
//!
//! You can also keep track of progress with [`set_message`]:
//!
//! ```rust
#![doc = include_str!("../examples/download.rs")]
//! ```
//!
//! [`Throbber`]: Throbber
//! [`start`]: Throbber::start
//! [`start_with_msg`]: Throbber::start_with_msg
//! [`set_message`]: Throbber::set_message
//! [`set_frames`]: Throbber::set_frames
//! [`success`]: Throbber::success

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
/// `[=    ]   [==   ]   [ ==  ]   [  == ]   [   ==]   [    =]   [   ==]   [  == ]   [ ==  ]   [==   ]`
pub const MOVE_EQ_LONG_F: [&str; 10] = [
    "[=    ]", "[==   ]", "[ ==  ]", "[  == ]", "[   ==]", "[    =]", "[   ==]", "[  == ]",
    "[ ==  ]", "[==   ]",
];
/// `[-    ]   [--   ]   [ --  ]   [  -- ]   [   --]   [    -]   [   --]   [  -- ]   [ --  ]   [--   ]`
pub const MOVE_MIN_LONG_F: [&str; 10] = [
    "[-    ]", "[--   ]", "[ --  ]", "[  -- ]", "[   --]", "[    -]", "[   --]", "[  -- ]",
    "[ --  ]", "[--   ]",
];

/// Representation of a throbber animation. It can start, succeed, fail or finish at any point.
///
/// Note that the Throbber thread gets spawned on the first call to [`start`](Throbber::start) or [`start_with_msg`](Throbber::start_with_msg). After that, the thread only ever gets parked.
/// If you want to end the thread, you must drop the Throbber object.
///
/// # Examples
///
/// ```rust
/// use std::thread;
/// use std::time::Duration;
/// use throbber::Throbber;
///
/// let mut throbber = Throbber::new(
///     "calculating stuff",
///     Duration::from_millis(50),
///     &throbber::ROTATE_F,
/// );
///
/// throbber.start();
///
/// // do stuff
/// thread::sleep(Duration::from_secs(5));
///
/// throbber.success("calculation successful");
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

impl Default for Throbber {
    /// # Default Values
    ///
    /// - message: `""`
    /// - interval: `Duration::from_millis(200)`
    /// - frames: `DEFAULT_F (⠋   ⠙   ⠹   ⠸   ⠼   ⠴   ⠦   ⠧   ⠇   ⠏)`
    fn default() -> Self {
        Self {
            anim: None,
            message: "".to_owned(),
            interval: Duration::from_millis(200),
            frames: &DEFAULT_F,
        }
    }
}

impl Drop for Throbber {
    fn drop(&mut self) {
        if let Some(anim) = self.anim.take() {
            anim.sender.send(ThrobberSignal::End).unwrap();
            anim.thread.thread().unpark();
            anim.thread.join().unwrap();
        }
    }
}

impl Throbber {
    /// Creates a new Throbber object.
    pub fn new<S: Into<String>>(
        message: S,
        interval: Duration,
        frames: &'static [&'static str],
    ) -> Self {
        Self {
            anim: None,
            message: message.into(),
            interval,
            frames,
        }
    }

    /// Sets the message displayed next to the throbber.
    pub fn message<S: Into<String>>(mut self, msg: S) -> Self {
        self.set_message(msg);
        self
    }

    /// Sets the message displayed next to the throbber.
    pub fn set_message<S: Into<String>>(&mut self, msg: S) {
        self.message = msg.into();
        if let Some(ref anim) = self.anim {
            anim.sender
                .send(ThrobberSignal::ChMsg(self.message.clone()))
                .unwrap();
            anim.thread.thread().unpark();
        }
    }

    /// Sets the animation frame interval, i.e. the time between frames.
    pub fn interval<D: Into<Duration>>(mut self, interval: D) -> Self {
        self.set_interval(interval);
        self
    }

    /// Sets the animation frame interval, i.e. the time between frames.
    pub fn set_interval<D: Into<Duration>>(&mut self, interval: D) {
        self.interval = interval.into();
        if let Some(ref anim) = self.anim {
            anim.sender
                .send(ThrobberSignal::ChInt(self.interval))
                .unwrap();
            anim.thread.thread().unpark();
        }
    }

    /// Sets the animation frames.
    pub fn frames(mut self, frames: &'static [&'static str]) -> Self {
        self.set_frames(frames);
        self
    }

    /// Sets the animation frames.
    pub fn set_frames(&mut self, frames: &'static [&'static str]) {
        self.frames = frames.into();
        if let Some(ref anim) = self.anim {
            anim.sender
                .send(ThrobberSignal::ChFrames(self.frames))
                .unwrap();
            anim.thread.thread().unpark();
        }
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
        let interval = self.interval;
        let frames = self.frames;
        let thread = thread::spawn(move || animation_thread(receiver, msg, interval, frames));

        self.anim = Some(ThrobberAnim { thread, sender });
    }

    /// Starts the animation with the specified `msg`.
    ///
    /// Equivalent to `throbber.set_message(msg); throbber.start();`.
    pub fn start_with_msg<S: Into<String>>(&mut self, msg: S) {
        self.set_message(msg);
        self.start();
    }

    /// Stops the current animation, leaving a blank line.
    pub fn finish(&mut self) {
        if let Some(ref anim) = self.anim {
            anim.sender.send(ThrobberSignal::Finish).unwrap();
            anim.thread.thread().unpark();
        }
    }

    /// Stops the current animation and prints `msg` as a *success message* (`✔`).
    pub fn success<'a, S: Into<String> + std::fmt::Display>(&mut self, msg: S) {
        if let Some(ref anim) = self.anim {
            anim.sender.send(ThrobberSignal::Succ(msg.into())).unwrap();
            anim.thread.thread().unpark();
        } else {
            println!("\x1B[2K\r✔ {}", msg);
        }
    }

    /// Stops the current animation and prints `msg` as a *fail message* (`✖`).
    ///
    /// This still prints to stdout, *not* stderr.
    pub fn fail<'a, S: Into<String>>(&mut self, msg: S) {
        let msg = msg.into();
        if let Some(ref anim) = self.anim {
            anim.sender.send(ThrobberSignal::Fail(msg)).unwrap();
            anim.thread.thread().unpark();
        } else {
            println!("\x1B[2K\r✖ {}", msg);
        }
    }
}

fn animation_thread<'a>(
    receiver: Receiver<ThrobberSignal>,
    mut msg: String,
    mut interval: Duration,
    mut frames: &'static [&'static str],
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
