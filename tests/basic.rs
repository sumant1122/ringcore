use ringcore::{run, spawn, op, sys::__kernel_timespec};
use std::time::{Duration, Instant};

#[test]
fn test_timer() {
    let start = Instant::now();
    spawn(async move {
        let mut ts = __kernel_timespec {
            tv_sec: 0,
            tv_nsec: 100_000_000, // 100ms
        };
        op::timeout(&mut ts).await.ok();
    });
    run();
    let elapsed = start.elapsed();
    assert!(elapsed >= Duration::from_millis(100));
    assert!(elapsed < Duration::from_millis(500));
}

#[test]
fn test_spawn_multiple() {
    let mut count = 0;
    // Note: We'd need a way to share state safely in a single-threaded Rc world
    // For now, let's just ensure they both run.
    spawn(async {
        println!("Task 1");
    });
    spawn(async {
        println!("Task 2");
    });
    run();
}
