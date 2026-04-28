use ringring::{run, spawn, op, sys::__kernel_timespec};
use std::time::{Duration, Instant};

async fn sleep(duration: Duration) {
    let mut ts = __kernel_timespec {
        tv_sec: duration.as_secs() as i64,
        tv_nsec: duration.subsec_nanos() as i64,
    };
    match op::timeout(&mut ts).await {
        Ok(_) => {},
        // ETIME (62) is expected for a timeout that completes
        Err(e) if e.raw_os_error() == Some(62) => {},
        Err(e) => panic!("Timeout failed: {:?}", e),
    }
}

fn main() {
    let start = Instant::now();
    spawn(async move {
        println!("Timer started: sleeping for 1 second...");
        sleep(Duration::from_secs(1)).await;
        println!("1 second passed at {:?}", start.elapsed());
        
        println!("Sleeping for 500ms...");
        sleep(Duration::from_millis(500)).await;
        println!("500ms passed. Done at {:?}", start.elapsed());
    });
    run();
}
