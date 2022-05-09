# Urusai

Urusai (JP: 煩い / noisy in Japanese) is a Discord Text-to-Speech bot written in Rust that reads text in a specific channel using the private TikTok TTS API and the [ttsmp3.com](https://ttsmp3.com/) API, and also omame's Remote SAPI Server API.

TTS configuration varies each server per user, per server.


# Usage

To use this bot, invite it to a server, then join a voice chat and run `tts!join` in the chat you would like it to read.

You can also make it leave the voice chat by running `tts!leave`.

To use another voice, run `tts!setvoice <voice>`.


# Building

[Install Rust and Cargo](https://www.rust-lang.org/), then install Opus and FFMPEG.

then run `cargo build --release`, or run it directly using `cargo run`.

# Running your own instance

Follow the building instructions above, then copy the release binary from target/release/urusai[.exe] to your own directory.

Install the dependencies (Opus and FFMPEG), then set these environment variables (or use the .env file):

```
DATABASE_URL=sqlite:database.db
DISCORD_TOKEN=<your token>
```