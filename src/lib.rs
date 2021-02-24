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

pub struct Loading {
    anim: Option<LoadingAnim>,
    message: String,
    duration: Duration,
    frames: &'static [&'static str],
}

enum LoadingSignal {
    Start,
    Succ(String),
    Fail(String),
    ChMsg(String),
    ChDur(Duration),
    ChFrames(&'static [&'static str]),
    End,
}

struct LoadingAnim {
    thread: JoinHandle<()>,
    sender: Sender<LoadingSignal>,
}

fn animation_thread(
    receiver: Receiver<LoadingSignal>,
    mut msg: String,
    mut frames: &'static [&'static str],
    mut duration: Duration,
) {
    let mut play_anim = true;
    let mut frame = 0;
    loop {
        match receiver.try_recv() {
            Ok(LoadingSignal::Start) => {
                play_anim = true;
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
            Ok(LoadingSignal::ChDur(new_dur)) => {
                duration = new_dur;
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
        thread::sleep(duration);
        frame = (frame + 1) % frames.len();
    }
}

impl Loading {
    pub fn new() -> Self {
        Self {
            anim: None,
            message: "".to_string(),
            duration: Duration::from_millis(200),
            frames: &DEFAULT_F,
        }
    }

    pub fn message(mut self, msg: String) -> Self {
        self.message = msg;
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
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

    pub fn change_duration(&mut self, duration: Duration) {
        if let Some(ref anim) = self.anim {
            anim.sender.send(LoadingSignal::ChDur(duration)).unwrap();
            anim.thread.thread().unpark();
        }
        self.duration = duration;
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
        let duration = self.duration;
        let thread = thread::spawn(move || animation_thread(receiver, msg, frames, duration));

        self.anim = Some(LoadingAnim { thread, sender });
    }

    pub fn start_with_msg(&mut self, msg: String) {
        self.change_message(msg);
        self.start();
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

    pub fn end(&mut self) -> thread::Result<()> {
        if let Some(anim) = self.anim.take() {
            anim.sender.send(LoadingSignal::End).unwrap();
            anim.thread.thread().unpark();
            anim.thread.join()?;
        }
        Ok(())
    }
}
