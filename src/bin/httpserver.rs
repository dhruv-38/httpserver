use std::io;
use std::sync::mpsc;

use httpfromsratch::internal::request::Request;
use httpfromsratch::internal::response::{
    StatusCode, Writer as ResponseWriter, get_default_headers,
};
use httpfromsratch::internal::server;

const PORT: u16 = 42069;

const BAD_REQUEST_HTML: &str = concat!(
    "<html>\n",
    "  <head>\n",
    "    <title>400 Bad Request</title>\n",
    "  </head>\n",
    "  <body>\n",
    "    <h1>Bad Request</h1>\n",
    "    <p>Your request honestly kinda sucked.</p>\n",
    "  </body>\n",
    "</html>"
);

const INTERNAL_SERVER_ERROR_HTML: &str = concat!(
    "<html>\n",
    "  <head>\n",
    "    <title>500 Internal Server Error</title>\n",
    "  </head>\n",
    "  <body>\n",
    "    <h1>Internal Server Error</h1>\n",
    "    <p>Okay, you know what? This one is on me.</p>\n",
    "  </body>\n",
    "</html>"
);

const SUCCESS_HTML: &str = concat!(
    "<html>\n",
    "  <head>\n",
    "    <title>200 OK</title>\n",
    "  </head>\n",
    "  <body>\n",
    "    <h1>Success!</h1>\n",
    "    <p>Your request was an absolute banger.</p>\n",
    "  </body>\n",
    "</html>"
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = server::serve(PORT, app_handler)?;

    println!("Server started on port {PORT}");

    let (shutdown_tx, shutdown_rx) = mpsc::channel();

    ctrlc::set_handler(move || {
        let _ = shutdown_tx.send(());
    })?;

    // Wait here until Ctrl+C causes the handler to send a message.
    shutdown_rx.recv()?;

    server.close()?;

    println!("Server gracefully stopped");

    Ok(())
}

fn app_handler(writer: &mut ResponseWriter<'_>, request: &Request) -> io::Result<()> {
    let request_line = request
        .request_line
        .as_ref()
        .expect("a parsed request must contain a request line");

    let (status_code, body) = match request_line.request_target.as_str() {
        "/yourproblem" => (StatusCode::BAD_REQUEST, BAD_REQUEST_HTML),

        "/myproblem" => (
            StatusCode::INTERNAL_SERVER_ERROR,
            INTERNAL_SERVER_ERROR_HTML,
        ),

        _ => (StatusCode::OK, SUCCESS_HTML),
    };

    let body_bytes = body.as_bytes();

    let mut headers = get_default_headers(body_bytes.len());
    headers.set("Content-Type", "text/html");

    writer.write_status_line(status_code)?;
    writer.write_headers(headers)?;
    writer.write_body(body_bytes)?;

    Ok(())
}
