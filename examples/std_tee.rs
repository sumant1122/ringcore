use std::env;
use std::fs::File;
use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <file1> <file2>", args[0]);
        return Ok(());
    }

    let mut f1 = File::create(&args[1])?;
    let mut f2 = File::create(&args[2])?;
    
    let mut buf = [0u8; 4096];
    let mut stdin = io::stdin();
    
    loop {
        let n = stdin.read(&mut buf)?;
        if n == 0 {
            break;
        }
        f1.write_all(&buf[..n])?;
        f2.write_all(&buf[..n])?;
    }
    
    Ok(())
}
