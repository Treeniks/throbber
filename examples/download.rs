use std::thread;
use std::time::Duration;
use throbber::Throbber;

fn main() {
    let mut throbber = Throbber::new(
        "Downloading file1 0%",
        Duration::from_millis(100),
        &throbber::ROTATE_F,
    );

    throbber.start();
    for i in 0..100 {
        throbber.set_message(format!("Downloading file1 {}%", i));
        thread::sleep(Duration::from_millis(30));
    }
    throbber.success("Downloaded file1");

    throbber.start_with_msg("Downloading file2 0%");
    for i in 0..69 {
        throbber.set_message(format!("Downloading file2 {}%", i));
        thread::sleep(Duration::from_millis(30));
    }
    throbber.fail("Download of file2 failed");
}
