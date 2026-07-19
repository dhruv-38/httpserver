use std::sync::mpsc;

use httpfromsratch::internal::server;

const PORT: u16 = 42069;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = server::serve(PORT)?;

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