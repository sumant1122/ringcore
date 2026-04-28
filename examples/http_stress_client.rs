use ringcore::{run, spawn, TcpStream};
use std::time::Instant;
use std::rc::Rc;
use std::cell::Cell;

async fn make_requests(id: usize, n: usize, done: Rc<Cell<usize>>) {
    for _ in 0..n {
        match TcpStream::connect("127.0.0.1:8080").await {
            Ok(stream) => {
                let request = "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
                if stream.write(request.as_bytes()).await.is_ok() {
                    let mut buf = [0u8; 1024];
                    let _ = stream.read(&mut buf).await;
                }
            }
            Err(e) => {
                eprintln!("Task {} connection error: {}", id, e);
            }
        }
    }
    done.set(done.get() + 1);
}

fn main() {
    let n_tasks = 200;
    let reqs_per_task = 5;
    let _total_expected = n_tasks;
    let done = Rc::new(Cell::new(0));

    println!("RingCore Stress Client: Spawning {} concurrent tasks ({} total requests)...", n_tasks, n_tasks * reqs_per_task);
    
    let start = Instant::now();
    for i in 0..n_tasks {
        let d = done.clone();
        spawn(async move {
            make_requests(i, reqs_per_task, d).await;
        });
    }

    run();
    
    let duration = start.elapsed();
    println!("RingCore Stress Client: Finished {} tasks in {:?}", done.get(), duration);
}
