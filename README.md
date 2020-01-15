![](https://github.com/HiruNya/dawn-wait/workflows/Rust/badge.svg)

[Documentation](https://hiru.dev/docs/dawn-wait/)

<!-- cargo-sync-readme start -->

This crate aims to provide an easy way to wait for new messages in a Discord channel.
This is meant to be used alongside the crates from the [dawn ecosystem](https://github.com/dawn-rs/dawn).

# Examples
To try out the examples in the `examples` directory,
a Discord token needs to be set as the environmental variable `DISCORD_TOKEN`.
This can be done by either creating a `.env` file or setting it from the shell.
```bash
export DISCORD_TOKEN="Discord.Token.Here"
cargo run --example basic
```
Then type in `!example` in a Discord channel which the bot has access to.

![Preview](https://imgur.com/ZlvcM6K.png)

# Note
This crate uses `chashmap` underneath for its concurrent data structure
which does not provide a way of polling the availability for a lock and therefore
the acquiring of a Read or Write lock cannot be asynchronous and must be blocking.
As there is a lock per channel, in the rare case that multiple listeners are listening on the same channel,
some operations *may* block.

<!-- cargo-sync-readme end -->