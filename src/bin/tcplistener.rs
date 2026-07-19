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
                let line = request
                    .request_line
                    .as_ref()
                    .expect("parsed request should contain a request line");

                println!("Request line:");
                println!("- Method: {}", line.method);
                println!("- Target: {}", line.request_target);
                println!("- Version: {}", line.http_version);

                println!("Headers:");
                for (key, value) in request.headers.iter() {
                    println!("- {}: {}", key, value);
                }

                println!("Body:");
                println!("{}", String::from_utf8_lossy(&request.body));
            }

            Err(error) => {
                eprintln!("Error parsing request: {}", error);
            }
        }

        println!("Connection closed");
    }

    Ok(())
}