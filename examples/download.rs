use std::thread;
use std::time::Duration;
use throbber::Throbber;

fn main() {
    let mut throbber = Throbber::new()
        .message("Downloading file1 0%".to_string())
        .frames(&throbber::ROTATE_F)
        .interval(Duration::from_millis(100));

    throbber.start();
    for i in 0..100 {
        throbber.change_message(format!("Downloading file1 {}%", i));
        thread::sleep(Duration::from_millis(30));
    }
    throbber.success("Downloaded file1".to_string());

    throbber.start_with_msg("Downloading file2 0%".to_string());
    for i in 0..69 {
        throbber.change_message(format!("Downloading file2 {}%", i));
        thread::sleep(Duration::from_millis(30));
    }
    throbber.fail("Download of file2 failed".to_string());

    throbber.end();
}
