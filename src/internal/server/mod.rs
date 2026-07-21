use crate::internal::request::{Request, request_from_reader};
use crate::internal::response::{
    StatusCode, get_default_headers, write_headers, write_status_line,
};
use std::io::{self, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[derive(Debug)]
pub struct HandlerError {
    pub status_code: StatusCode,
    pub message: String,
}

impl HandlerError {
    pub fn new(status_code: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status_code,
            message: message.into(),
        }
    }
}

pub type Handler = fn(writer: &mut dyn Write, request: &Request) -> Result<(), HandlerError>;

pub struct Server {
    is_closed: Arc<AtomicBool>,
    listener_thread: Option<JoinHandle<()>>,
}

pub fn serve(port: u16, handler: Handler) -> io::Result<Server> {
    let listener = TcpListener::bind(("0.0.0.0", port))?;

    // We use non-blocking mode so the listener thread can regularly check
    // whether close() has requested that the server stop.
    listener.set_nonblocking(true)?;

    let is_closed = Arc::new(AtomicBool::new(false));
    let thread_is_closed = Arc::clone(&is_closed);

    let listener_thread = thread::spawn(move || {
        listen(listener, thread_is_closed, handler);
    });

    Ok(Server {
        is_closed,
        listener_thread: Some(listener_thread),
    })
}

fn listen(listener: TcpListener, is_closed: Arc<AtomicBool>, handler: Handler) {
    while !is_closed.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _address)) => {
                thread::spawn(move || {
                    if let Err(error) = handle(stream, handler) {
                        eprintln!("Error handling connection: {error}");
                    }
                });
            }

            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                // No connection is currently waiting.
                thread::sleep(Duration::from_millis(50));
            }

            Err(error) => {
                if !is_closed.load(Ordering::Acquire) {
                    eprintln!("Error accepting connection: {error}");
                }

                break;
            }
        }
    }
}

fn handle(mut stream: TcpStream, handler: Handler) -> io::Result<()> {
    let request = request_from_reader(&mut stream)?;

    let mut response_body = Vec::new();

    match handler(&mut response_body, &request) {
        Ok(()) => {
            write_status_line(&mut stream, StatusCode::OK)?;

            let headers = get_default_headers(response_body.len());
            write_headers(&mut stream, &headers)?;

            stream.write_all(&response_body)?;
        }

        Err(handler_error) => {
            write_handler_error(&mut stream, &handler_error)?;
        }
    }

    stream.flush()?;

    Ok(())
}

fn write_handler_error(writer: &mut impl Write, error: &HandlerError) -> io::Result<()> {
    write_status_line(writer, error.status_code)?;

    let headers = get_default_headers(error.message.as_bytes().len());
    write_headers(writer, &headers)?;

    writer.write_all(error.message.as_bytes())
}

impl Server {
    pub fn close(&mut self) -> io::Result<()> {
        self.is_closed.store(true, Ordering::Release);

        if let Some(listener_thread) = self.listener_thread.take() {
            listener_thread
                .join()
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "listener thread panicked"))?;
        }

        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
