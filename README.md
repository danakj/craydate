# Set Up
1. Install playdate SDK: https://play.date/dev/
1. Set PLAYDATE_SDK_PATH env var to point at the SDK install dir.
1. Install clang: https://releases.llvm.org/download.html
    * For windows look for a .exe file on the GitHub release page. It doesn't say clang in the name but it's there.

# Building for device

1. rustup target add thumbv7em-none-eabihf
1. cargo build --release --lib --target thumbv7em-none-eabihf
1. The build will print `"dropping unsupported crate type cdylib"` warning. This
   is normal and expected when building for the device.

# Status

## Verion 1.10.0 Support

[Api](https://sdk.play.date/1.9.3/Inside%20Playdate%20with%20C.html#_api_reference) coverage:
- 6.1 Utility **[ DONE ]**
- 6.2 Audio **[ DONE ]** (bugs filed: some incomplete C APIs and completion callbacks run on the wrong thread and crash)
- 6.3 Display **[ DONE ]**
- 6.4 Filesystem **[ DONE ]**
- 6.5 Graphics **[ DONE ]** (except BitmapTables due to C API being incomplete)
- 6.6 Video **[ DONE ]**
- 6.7 Input **[ DONE ]**
- 6.8 Device Auto Lock **[ DONE ]**
- 6.9 System Sounds. **[ DONE ]**
- 6.10 JSON **[ WONTFIX: Use [postcard](https://docs.rs/postcard/latest/postcard/) instead ]**
- 6.11 Lua **[ WONTFIX: No current plan to support Rust backend for Lua games ]**
- 6.12 Sprites  **[ WONTFIX: No current plan to support Sprites ]**

There are still some TODOs around for a few missing functions.
