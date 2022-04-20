use super::file_timestamp::FileTimestamp;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileStat {
  pub is_folder: bool,
  pub size: u32,
  pub modified: FileTimestamp,
}
