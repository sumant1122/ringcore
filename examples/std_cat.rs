use std::env;
use std::fs::File;
use std::io::{self, Read, Write};

fn cat(path: String) -> io::Result<()> {
    let mut file = File::open(path)?;
    let mut buf = [0u8; 4096];
    let mut stdout = io::stdout();
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        stdout.write_all(&buf[..n])?;
    }
    stdout.flush()?;
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file>", args[0]);
        return;
    }

    let path = args[1].clone();
    if let Err(e) = cat(path) {
        eprintln!("Error: {}", e);
    }
}
