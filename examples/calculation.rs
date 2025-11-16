use std::thread;
use std::time::Duration;
use throbber::Throbber;

fn main() {
    let mut throbber = Throbber::default().message("calculating stuff");

    throbber.start();
    // do stuff
    thread::sleep(Duration::from_secs(2));
    throbber.success("Success");

    throbber.start_with_msg("calculating more stuff");
    // do other stuff
    thread::sleep(Duration::from_secs(2));
    throbber.fail("Fail");
}
