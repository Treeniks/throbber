This crate serves as an alternative to [loading](https://crates.io/crates/loading). It is used to display a throbber animation in the terminal while other calculations are done in the main program.

# Usage

First, add this to your Cargo.toml:

```toml
[dependencies]
throbber = "0.1"
```

Then:

```rust
use throbber::Throbber;
use std::thread;
use std::time::Duration;

fn main() {
	let mut throbber = Throbber::new().message("calculating stuff".to_string());

	throbber.start();
	thread::sleep(Duration::from_secs(5));
	throbber.success("calculation was successful".to_string());

	throbber.start_with_msg("calculating more stuff".to_string());
	thread::sleep(Duration::from_secs(3));
	throbber.fail("calculation failed".to_string());

	throbber.end();
}
```
