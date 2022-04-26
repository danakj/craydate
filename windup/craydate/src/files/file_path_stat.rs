use super::file_path_timestamp::FilePathTimestamp;

/// Information about a file path in the filesystem.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FilePathStat {
  /// The path refers to a folder.
  Folder {
    /// When the folder was last modified, which occurs when a file is created or deleted.
    modified: FilePathTimestamp,
  },
  /// The path refers to a file.
  File {
    /// The size of the file in bytes.
    size: u32,
    /// When the file was last modified.
    modified: FilePathTimestamp,
  },
}
