# Set Up
1. rustup target add armv7a-none-eabi (needed for Playdate device, not the simulator)
1. Install playdate SDK: https://play.date/dev/
1. Set PLAYDATE_SDK_PATH env var to point at the SDK install dir.
1. Install clang: https://releases.llvm.org/download.html
    * For windows look for a .exe file on the GitHub release page. It doesn't say clang in the name but it's there.

# Status

## Verion 1.9.3 Support

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

# Adding to OS target
1. Make a new top-level crate, e.g. simulator-win
1. Set the target triple in the crate's `.cargo/config.toml` file, e.g.
   `simulator-win/.cargo/config.toml`.
1. Add a task to build and check it in `.vscode/tasks.json`. Then Ctrl+Shift+B will give you
   the option to build it.
1. Also add tasks run build and run the make_pdx and run_simulator bins, if they apply, in
   `.vscode/tasks.json`. Then Ctrl+Shift+B will give you the option to run the project
   in the simulator found at `$PLAYDATE_SDK_PATH`.
1. Add it to `"rust-analyzer.linkedProjects"` in `.vscode/settings.json`. Then rust-analyzer will
   index it.

# TODOs
- Turn all `*mut T` into `NonNull<T>` where it makes sense.
- Add a `Self::fns() -> &'static capithing` fn to all the types.
- Add a `Self::cptr(&self) -> *mut T` to all the types.