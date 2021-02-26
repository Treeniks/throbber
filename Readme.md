# Throbber

[![Crates.io](https://img.shields.io/crates/v/throbber?style=flat-square)](https://crates.io/crates/throbber)
[![docs.rs](https://img.shields.io/docsrs/throbber?style=flat-square)](https://docs.rs/throbber)

This crate serves as an alternative to [loading](https://crates.io/crates/loading). It is used to display a throbber animation in the terminal while other calculations are done in the main program.

![Throbber Preview](https://user-images.githubusercontent.com/56131826/109326392-68c28b00-7857-11eb-8e8d-dd576c868e7f.gif "Throbber Preview")

## Usage

Add this to your Cargo.toml:

```toml
[dependencies]
throbber = "0.1"
```

To display a throbber animation, first create a Throbber object:

```rust
let mut throbber = Throbber::new();
```

You can also customize certain settings like the displayed animation and the displayed message:

```rust
let mut throbber = Throbber::new()
    .message("calculating stuff".to_string())
    .frames(&throbber::MOVE_EQ_F); // this crate comes with a few predefined animations
                                   // see the Constants section
```

Then you can simply call `start()` wherever you want to start the animation and a *finish function* like `success()` where you want to stop it.

```rust
throbber.start();
// do calculations
throbber.success("calculations successful!".to_string());
```

After which you can call `start()` or `start_with_msg(String)` again to start the animation again.
You can also change everything you could customize during the Throbber object creation, e. g. with `change_message(String)` and `change_frames(String)`. This also works while an animation is running.

If you don't intend to start another animation, you should drop the Throbber object with `end()`. This action also ends the underlying thread:

```rust
throbber.end();
```

## Examples

This is the example from the preview above and can be run with `cargo run --example calculation`:

```rust
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
```

You can also keep track of progress with `change_message(String)`. This can be run with `cargo run --example download`:

```rust
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
```
