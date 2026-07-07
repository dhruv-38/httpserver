use std::net::TcpListener;

use httpfromsratch::internal::request::request_from_reader;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:42069")?;

    println!("Listening on 127.0.0.1:42069");

    for stream in listener.incoming() {
        let stream = stream?;

        println!("Connection accepted");

        match request_from_reader(stream) {
            Ok(request) => {
                let line = request.request_line.unwrap();

                println!("Request line:");
                println!("- Method: {}", line.method);
                println!("- Target: {}", line.request_target);
                println!("- Version: {}", line.http_version);
            }

            Err(e) => {
                eprintln!("Error parsing request: {}", e);
            }
        }

        println!("Connection closed");
    }

    Ok(())
}