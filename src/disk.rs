//! Disk-related types and helper functions.

use super::{GptConfig, GptDisk, GptError};
use std::{fmt, fs, io, path};

/// Default size of a logical sector (bytes).
pub const DEFAULT_SECTOR_SIZE: LogicalBlockSize = LogicalBlockSize::Lb512;

/// Valid maximum sector size in bytes according to gpt specification
/// 4gb is still valid apparently. 
/// who the FUCK came up with this.
pub const MAX_SECTOR_SIZE: u64 = u64::pow(2,32);

/// Logical block/sector size of a GPT disk.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogicalBlockSize {
    /// 512 bytes.
    Lb512,
    /// 4096 bytes.
    Lb4096,

    /// Other unusual block sizes.
    Other(u64)
}

impl LogicalBlockSize {
    /// Returns the logical block size as a `usize`.
    pub const fn as_usize(&self) -> usize {
        match self {
            LogicalBlockSize::Lb512 => 512,
            LogicalBlockSize::Lb4096 => 4096,
            LogicalBlockSize::Other(block_size) => *block_size as usize
        }
    }

    /// Returns the logical block size as a `u64`.
    pub const fn as_u64(&self) -> u64 {
        match self {
            LogicalBlockSize::Lb512 => 512,
            LogicalBlockSize::Lb4096 => 4096,
            LogicalBlockSize::Other(block_size) => *block_size
        }
    }
}

impl From<LogicalBlockSize> for u64 {
    fn from(lb: LogicalBlockSize) -> u64 {
        lb.as_u64()
    }
}

impl From<LogicalBlockSize> for usize {
    fn from(lb: LogicalBlockSize) -> usize {
        lb.as_usize()
    }
}

impl TryFrom<u64> for LogicalBlockSize {
    type Error = io::Error;
    fn try_from(block_size: u64) -> Result<Self, Self::Error> {
        match block_size {
            512 => Ok(LogicalBlockSize::Lb512),
            4096 => Ok(LogicalBlockSize::Lb4096),
            512..=MAX_SECTOR_SIZE => Ok(LogicalBlockSize::Other(block_size)),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Logical block size {} is NOT in the range 512B-4GB", block_size),
            )),
        }
    }
}

impl fmt::Display for LogicalBlockSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LogicalBlockSize::Lb512 => write!(f, "512"),
            LogicalBlockSize::Lb4096 => write!(f, "4096"),
            LogicalBlockSize::Other(block_size) => write!(f, "{}", block_size),
        }
    }
}

/// Open and read a GPT disk, using default configuration options.
///
/// ## Example
///
/// ```rust,no_run
/// let gpt_disk = gpt_toolbox::disk::read_disk("/dev/sdz").unwrap();
/// println!("{:#?}", gpt_disk);
/// ```
pub fn read_disk(diskpath: impl AsRef<path::Path>) -> Result<GptDisk<fs::File>, GptError> {
    let cfg = GptConfig::new();
    cfg.open(diskpath)
}
