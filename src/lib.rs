#![warn(missing_docs)]
//! This crate aims to provide an easy way to wait for new messages in a Discord channel.
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
//!
//! ![Preview](https://imgur.com/ZlvcM6K.png)
//!
//! # Note
//! This crate uses `chashmap` underneath for its concurrent data structure
//! which does not provide a way of polling the availability for a lock and therefore
//! the acquiring of a Read or Write lock cannot be asynchronous and must be blocking.
//! As there is a lock per channel, in the rare case that multiple listeners are listening on the same channel,
//! some operations *may* block.

mod listener;
mod wait;

pub use listener::Listener;
pub use wait::{WaitFor, WaitForMultiple};

#[cfg(test)]
mod tests {
    #[test]
    fn cleanup() {
        use dawn::model::id::ChannelId;
        use crate::Listener;
        
        let listener = Listener::default();
        {
            let _wait1 = listener.wait_for(ChannelId::default(), |_| true);
            let _wait2 = listener.wait_for(ChannelId::default(), |_| true);
            
            // 2 listener items were created for the *same* channel.
            assert_eq!(listener.items.len(), 1);
            assert_eq!(listener.items.get(&ChannelId::default()).expect("Channel doesn't exit").len(), 2);
        }

        // When they are dropped, the listener should be empty.
        assert!(listener.items.is_empty());
    }
}
