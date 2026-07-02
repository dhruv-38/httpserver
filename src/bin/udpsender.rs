use std::io::{self, Write};
use std::net::UdpSocket;

fn main() -> io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:0")?;

    socket.connect("127.0.0.1:42069")?;

    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        print!("> ");
        io::stdout().flush()?;

        input.clear();

        let bytes_read = stdin.read_line(&mut input)?;

        if bytes_read == 0 {
            break;
        }

        socket.send(input.as_bytes())?;
    }

    Ok(())
}