use std::io::Write;
use std::sync::mpsc;

use httpfromsratch::internal::request::Request;
use httpfromsratch::internal::response::StatusCode;
use httpfromsratch::internal::server::{self, HandlerError};

const PORT: u16 = 42069;

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

fn app_handler(
    writer: &mut dyn Write,
    request: &Request,
) -> Result<(), HandlerError> {
    let request_line = request.request_line.as_ref().ok_or_else(|| {
        HandlerError::new(
            StatusCode::BAD_REQUEST,
            "Missing request line\n",
        )
    })?;

    match request_line.request_target.as_str() {
        "/yourproblem" => {
            Err(HandlerError::new(
                StatusCode::BAD_REQUEST,
                "Your problem is not my problem\n",
            ))
        }

        "/myproblem" => {
            Err(HandlerError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Woopsie, my bad\n",
            ))
        }

        _ => {
            writer
                .write_all(b"All good, frfr\n")
                .map_err(|_| {
                    HandlerError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to write response\n",
                    )
                })?;

            Ok(())
        }
    }
}