use std::thread;
use std::time::Duration;
use throbber::Throbber;

fn main() {
    let mut throbber = Throbber::new().message("calculating stuff".to_string());

    throbber.start();
    // do stuff
    thread::sleep(Duration::from_secs(2));
    throbber.success("Success".to_string());

    throbber.start_with_msg("calculating more stuff".to_string());
    // do other stuff
    thread::sleep(Duration::from_secs(2));
    throbber.fail("Fail".to_string());

    throbber.end();
}
