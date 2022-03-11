#![no_std]

fn windup_game_goes_here() {
    playdate::CStr::from_bytes_with_nul(b"hi\0");
}