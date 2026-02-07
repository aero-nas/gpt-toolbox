//! Disk-related types and helper functions.

use super::{GptConfig, GptDisk, GptError};
use std::{fmt, fs, io, path};

use nix::{libc::ioctl, errno::Errno};
use std::os::unix::io::AsRawFd;

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

/// Struct for dk_minfo defined in 
/// https://www.unix.com/man-page/opensolaris/7I/dkio/
#[cfg(any(target_os = "solaris", target_os = "illumos"))]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct dk_minfo {
    /// Media type or profile info
    pub dki_media_type: u32,

    /// Logical blocksize of media
    pub dki_lbsize: u32,

    /// Capacity as # of dki_lbsize blks
    pub dki_capacity: u64,
}

/// Get sector size
/// Supports: 
/// Linux 
/// BSD (untested)
/// Solaris/Illumos (untested)
/// MacOS (untested)
/// 
/// unsafe because it uses nix::libc::ioctl
#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",

    target_os = "solaris",
    target_os = "illumos",
    
    target_os = "linux",
    
    target_os = "macos",
))]
pub unsafe fn get_block_size(diskpath: &str) -> Result<LogicalBlockSize, GptError> {
    let file = fs::File::open(diskpath)?;
    let fd = file.as_raw_fd();

    let mut block_size: u64 = 0;

    let result = unsafe {
        // https://unix.stackexchange.com/a/52222
        #[cfg(target_os = "linux")] 
        {
            ioctl(fd, nix::libc::BLKSSZGET, &mut block_size)
        }

        // https://github.com/Kostassoid/lethe/blob/d1cdf1b926bba8b262d1f6d901550ba5287ae727/src/storage/nix/macos.rs#L37
        #[cfg(target_os = "macos")] 
        {   
            let mut block_size_u32: u32 = 0; 

            let res = ioctl(fd, nix::libc::DKIOCGETBLOCKSIZE, &mut block_size_u32);

            if res == 0 {
                block_size = block_size_u32 as u64;
            }
            
            res
        }
        
        // https://man.netbsd.org/disk.9#DISK%20IOCTLS
        #[cfg(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd"
        ))]
        { 
            ioctl(fd, nix::libc::DIOCGSECTORSIZE, &mut block_size) 
        }

        // https://www.unix.com/man-page/opensolaris/7I/dkio/
        #[cfg(any(target_os = "solaris", target_os = "illumos"))]
        { 
            let mut minfo = dk_minfo {
                dki_lbsize: 0,
                dki_capacity: 0,
                dki_media_type: 0,
            };

            let res = ioctl(fd, nix::libc::DKIOCGMEDIAINFO, &mut minfo);

            if res == 0 {
                block_size = minfo.dki_lbsize as u64;
            }

            res
        }
    };

    if result == -1 {
        return Err(GptError::Io(io::Error::from(Errno::last())))
    }

    match block_size {
        512 => Ok(LogicalBlockSize::Lb512),
        4096 => Ok(LogicalBlockSize::Lb4096),
        _ => Ok(LogicalBlockSize::Other(block_size))
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
