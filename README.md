# Set Up
1. rustup target add arm-linux-androideabi
1. Install playdate SDK: https://play.date/dev/
1. Set PLAYDATE_SDK_PATH env var to point at the SDK install dir
    * Replace <stdlib.h> and <string.h> with <stddef.h> in C_API/pd_api.h and C_API/pd_api/pd_api_json.h
1. Install clang: https://releases.llvm.org/download.html
    * For windows look for a .exe file on the GitHub release page. It doesn't say clang in the name but it's there.
