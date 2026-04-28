//! Asynchronous File System operations.

use crate::op;
use std::ffi::CString;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::Path;

/// An asynchronous file handle.
pub struct File {
    fd: RawFd,
    offset: u64,
}

impl AsRawFd for File {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl File {
    /// Open a file in read-only mode.
    pub async fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = CString::new(path.as_ref().as_os_str().as_bytes())?;
        let fd = op::openat(libc::AT_FDCWD, path.as_ptr(), libc::O_RDONLY, 0).await?;
        Ok(File { fd, offset: 0 })
    }

    /// Create a new file or truncate an existing one.
    pub async fn create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = CString::new(path.as_ref().as_os_str().as_bytes())?;
        let flags = libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC;
        let fd = op::openat(libc::AT_FDCWD, path.as_ptr(), flags, 0o644).await?;
        Ok(File { fd, offset: 0 })
    }

    /// Read some bytes from the file into the given buffer.
    /// 
    /// This updates the internal file offset upon success.
    pub async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let res = op::read(self.fd, buf.as_mut_ptr(), buf.len() as u32, self.offset).await?;
        let n = res as usize;
        self.offset += n as u64;
        Ok(n)
    }

    /// Write a buffer into the file.
    /// 
    /// This updates the internal file offset upon success.
    pub async fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let res = op::write(self.fd, buf.as_ptr(), buf.len() as u32, self.offset).await?;
        let n = res as usize;
        self.offset += n as u64;
        Ok(n)
    }

    /// Close the file.
    pub async fn close(self) -> io::Result<()> {
        op::close(self.fd).await?;
        std::mem::forget(self);
        Ok(())
    }
}

impl Drop for File {
    fn drop(&mut self) {
        let _ = unsafe { libc::close(self.fd) };
    }
}
