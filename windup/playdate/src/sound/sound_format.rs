use crate::ctypes_enums::SoundFormat;

/// Returns whether a SoundFormat is stereo. Otherwise, it is mono.
pub fn sound_format_is_stereo(sound_format: SoundFormat) -> bool {
  sound_format.0 & 1 == 1
}

/// Returns whether a SoundFormat is 16 bit. Otherwise, it is 8 bit.
pub fn sound_format_is_16_bit(sound_format: SoundFormat) -> bool {
  sound_format.0 >= SoundFormat::kSound16bitMono.0
}

/// Returns the number of bytes per sample frame for the SoundFormat.
pub fn sound_format_bytes_per_frame(sound_format: SoundFormat) -> usize {
  let stereo = if sound_format_is_stereo(sound_format) {
    2
  } else {
    1
  };
  let bytes = if sound_format_is_16_bit(sound_format) {
    2
  } else {
    1
  };
  stereo * bytes
}
