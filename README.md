# Set Up
1. rustup target add armv7a-none-eabi
1. Install playdate SDK: https://play.date/dev/
1. Set PLAYDATE_SDK_PATH env var to point at the SDK install dir
    * Or set it in [env] section of Cargo config file.
1. Install clang: https://releases.llvm.org/download.html
    * For windows look for a .exe file on the GitHub release page. It doesn't say clang in the name but it's there.

# Adding to OS target
1. Make a new top-level crate, e.g. simulator-win
1. Set the target triple in the crate's `.cargo/config.toml` file, e.g.
   `simulator-win/.cargo/config.toml`.
1. Add a task to build and check it in `.vscode/tasks.json`. Then Ctrl+Shift+B will give you
   the option to build it.
1. Add it to `"rust-analyzer.linkedProjects"` in `.vscode/settings.json`. Then rust-analyzer will
   index it.
