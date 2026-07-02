use std::io::{self, Read};
use std::net::TcpListener;
use std::sync::mpsc::{self, Receiver};
use std::thread;

fn get_lines_channel( mut reader: impl Read + Send + 'static) -> Receiver<String> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut buffer = [0u8; 8];
        let mut current_line = String::new();
        loop {
            let bytes_read = match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
            };
            let chunk = String::from_utf8_lossy(&buffer[..bytes_read]);
            let parts: Vec<&str> = chunk.split('\n').collect();
            for part in &parts[..parts.len() - 1] {
                let line = format!("{}{}", current_line, part);

                if tx.send(line).is_err() {
                    return;
                }

                current_line.clear();
            }
            current_line.push_str(parts[parts.len() - 1]);
        }
        if !current_line.is_empty() {
            let _ = tx.send(current_line);
        }
        // reader is closed automatically here when dropped
        // tx is closed automatically here when dropped
    });
    rx
}


fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:42069")?;
    println!("Listening on 127.0.0.1:42069");

    for stream in listener.incoming() {
        let stream = stream?;

        println!("Connection accepted");

        let lines = get_lines_channel(stream);

        for line in lines {
            println!("{}", line);
        }

        println!("Connection closed");
    }

    Ok(())
}