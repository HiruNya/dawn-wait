#![warn(missing_docs)]
//! This crate aims to provide an easy API to wait for new messages in a channel.
//! This is meant to be used alongside the crates from the [dawn ecosystem](https://github.com/dawn-rs/dawn).
//!
//! # Examples
//! To try out the examples in the `examples` directory,
//! a Discord token needs to be set as the environmental variable `DISCORD_TOKEN`.
//! This can be done by either creating a `.env` file or setting it from the shell.
//! ```bash
//! export DISCORD_TOKEN="Discord.Token.Here"
//! cargo run --example basic
//! ```
//! Then type in `!example` in a Discord channel which the bot has access to.

mod listener;
mod wait;

pub use listener::Listener;
pub use wait::{WaitFor, WaitForMultiple};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
