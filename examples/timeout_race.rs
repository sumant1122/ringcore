use ringcore::{run, spawn, op, sys::__kernel_timespec};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::io;

enum Winner<T1, T2> {
    Left(T1),
    Right(T2),
}

struct Select<F1, F2>(F1, F2);

impl<F1, F2, T1, T2> Future for Select<F1, F2>
where
    F1: Future<Output = T1> + Unpin,
    F2: Future<Output = T2> + Unpin,
{
    type Output = Winner<T1, T2>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Poll::Ready(val) = Pin::new(&mut self.0).poll(cx) {
            return Poll::Ready(Winner::Left(val));
        }
        if let Poll::Ready(val) = Pin::new(&mut self.1).poll(cx) {
            return Poll::Ready(Winner::Right(val));
        }
        Poll::Pending
    }
}

async fn race() -> io::Result<()> {
    // Let's just race two Ops for simplicity in the demo
    let mut ts1 = __kernel_timespec { tv_sec: 2, tv_nsec: 0 };
    let mut ts2 = __kernel_timespec { tv_sec: 1, tv_nsec: 0 };
    
    let op1 = op::timeout(&mut ts1);
    let op2 = op::timeout(&mut ts2);
    
    let id1 = op1.id;
    let id2 = op2.id;
    
    println!("Racing two timeouts: 2s vs 1s");
    
    match Select(op1, op2).await {
        Winner::Left(_) => {
            println!("2s timeout won? (Should not happen)");
            let _ = op::cancel(id2).await;
        }
        Winner::Right(_) => {
            println!("1s timeout won! Cancelling the 2s one.");
            let _ = op::cancel(id1).await;
        }
    }
    
    Ok(())
}

fn main() {
    spawn(async {
        if let Err(e) = race().await {
            eprintln!("Race error: {}", e);
        }
    });
    run();
}
