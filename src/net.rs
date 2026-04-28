//! Asynchronous Networking operations.

use crate::op;
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::os::unix::io::{RawFd, AsRawFd};

/// A TCP socket server, listening for connections.
pub struct TcpListener {
    fd: RawFd,
}

impl AsRawFd for TcpListener {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl TcpListener {
    /// Create a TcpListener from a raw file descriptor.
    pub fn from_raw_fd(fd: RawFd) -> Self {
        TcpListener { fd }
    }

    /// Bind to an address and start listening for connections.
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let addr = addr.to_socket_addrs()?.next().ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            "could not resolve address",
        ))?;

        let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let mut sa: libc::sockaddr_in = unsafe { std::mem::zeroed() };
        sa.sin_family = libc::AF_INET as u16;
        sa.sin_port = addr.port().to_be();
        if let SocketAddr::V4(v4) = addr {
            sa.sin_addr.s_addr = u32::from_ne_bytes(v4.ip().octets());
        } else {
            return Err(io::Error::new(io::ErrorKind::Unsupported, "IPv6 not supported"));
        }

        unsafe {
            let opt: i32 = 1;
            libc::setsockopt(fd, libc::SOL_SOCKET, libc::SO_REUSEADDR, &opt as *const _ as *const libc::c_void, std::mem::size_of::<i32>() as u32);
            
            if libc::bind(fd, &sa as *const _ as *const libc::sockaddr, std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t) < 0 {
                return Err(io::Error::last_os_error());
            }
            if libc::listen(fd, 128) < 0 {
                return Err(io::Error::last_os_error());
            }
        }

        Ok(TcpListener { fd })
    }

    /// Accept a new incoming connection.
    pub async fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let mut addr: libc::sockaddr_in = unsafe { std::mem::zeroed() };
        let mut len = std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
        
        let client_fd = op::accept(self.fd, &mut addr as *mut _ as *mut libc::sockaddr, &mut len).await?;
        
        let ip = std::net::Ipv4Addr::from(addr.sin_addr.s_addr.to_ne_bytes());
        let port = u16::from_be(addr.sin_port);
        let socket_addr = SocketAddr::new(std::net::IpAddr::V4(ip), port);
        
        Ok((TcpStream { fd: client_fd }, socket_addr))
    }
}

impl Drop for TcpListener {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd); }
    }
}

/// A TCP stream between a local and a remote socket.
pub struct TcpStream {
    fd: RawFd,
}

impl AsRawFd for TcpStream {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl TcpStream {
    /// Create a TcpStream from a raw file descriptor.
    pub fn from_raw_fd(fd: RawFd) -> Self {
        TcpStream { fd }
    }

    /// Connect to a remote address.
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let addr = addr.to_socket_addrs()?.next().ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            "could not resolve address",
        ))?;

        let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let mut sa: libc::sockaddr_in = unsafe { std::mem::zeroed() };
        sa.sin_family = libc::AF_INET as u16;
        sa.sin_port = addr.port().to_be();
        if let SocketAddr::V4(v4) = addr {
            sa.sin_addr.s_addr = u32::from_ne_bytes(v4.ip().octets());
        } else {
            return Err(io::Error::new(io::ErrorKind::Unsupported, "IPv6 not supported"));
        }

        op::connect(
            fd,
            &sa as *const _ as *const libc::sockaddr,
            std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
        )
        .await?;

        Ok(TcpStream { fd })
    }

    /// Read data from the stream into the provided buffer.
    pub async fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let res = op::read(self.fd, buf.as_mut_ptr(), buf.len() as u32, 0).await?;
        Ok(res as usize)
    }

    /// Write data from the provided buffer into the stream.
    pub async fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let res = op::write(self.fd, buf.as_ptr(), buf.len() as u32, 0).await?;
        Ok(res as usize)
    }
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd); }
    }
}
