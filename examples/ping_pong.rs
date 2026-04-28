use ringcore::{run, spawn, op};
use std::io;

async fn ping(fd: i32) -> io::Result<()> {
    let mut buf = [0u8; 4];
    for i in 0..5 {
        let msg = format!("P{:02}", i);
        println!("Ping sending: {}", msg);
        op::write(fd, msg.as_ptr(), 3, 0).await?;
        
        let n = op::read(fd, buf.as_mut_ptr(), 3, 0).await?;
        println!("Ping received: {}", String::from_utf8_lossy(&buf[..n as usize]));
    }
    Ok(())
}

async fn pong(fd: i32) -> io::Result<()> {
    let mut buf = [0u8; 4];
    for _ in 0..5 {
        let n = op::read(fd, buf.as_mut_ptr(), 3, 0).await?;
        let msg = String::from_utf8_lossy(&buf[..n as usize]);
        println!("Pong received: {}", msg);
        
        let reply = format!("R{}", &msg[1..]);
        println!("Pong sending reply: {}", reply);
        op::write(fd, reply.as_ptr(), 3, 0).await?;
    }
    Ok(())
}

fn main() {
    let mut fds = [0i32; 2];
    unsafe {
        if libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()) < 0 {
            panic!("socketpair failed: {}", io::Error::last_os_error());
        }
    }

    spawn(async move {
        if let Err(e) = ping(fds[0]).await {
            eprintln!("Ping error: {}", e);
        }
    });

    spawn(async move {
        if let Err(e) = pong(fds[1]).await {
            eprintln!("Pong error: {}", e);
        }
    });

    run();
}
