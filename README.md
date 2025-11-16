# Throbber

[![Crates.io](https://img.shields.io/crates/v/throbber)](https://crates.io/crates/throbber)
[![docs.rs](https://docs.rs/throbber/badge.svg)](https://docs.rs/throbber)
[![GitHub last commit](https://img.shields.io/github/last-commit/Treeniks/throbber)](https://github.com/Treeniks/throbber)
[![License](https://img.shields.io/github/license/Treeniks/throbber)](https://github.com/Treeniks/throbber/blob/master/LICENSE)

This crate serves as an alternative to [loading](https://crates.io/crates/loading). It is used to display a throbber animation in the terminal while other calculations are done in the main program.

![Throbber Preview](https://user-images.githubusercontent.com/56131826/109326392-68c28b00-7857-11eb-8e8d-dd576c868e7f.gif "Throbber Preview")

# [Docs](https://docs.rs/throbber/latest/throbber/index.html)

# Example

```rust
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
```
