use std::io::{self, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct Server {
    is_closed: Arc<AtomicBool>,
    listener_thread: Option<JoinHandle<()>>,
}

pub fn serve(port: u16) -> io::Result<Server> {
    let listener = TcpListener::bind(("0.0.0.0", port))?;

    // We use non-blocking mode so the listener thread can regularly check
    // whether close() has requested that the server stop.
    listener.set_nonblocking(true)?;

    let is_closed = Arc::new(AtomicBool::new(false));
    let thread_is_closed = Arc::clone(&is_closed);

    let listener_thread = thread::spawn(move || {
        listen(listener, thread_is_closed);
    });

    Ok(Server {
        is_closed,
        listener_thread: Some(listener_thread),
    })
}

fn listen(listener: TcpListener, is_closed: Arc<AtomicBool>) {
    while !is_closed.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _address)) => {
                thread::spawn(move || {
                    if let Err(error) = handle(stream) {
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

fn handle(mut stream: TcpStream) -> io::Result<()> {
    let response = concat!(
        "HTTP/1.1 200 OK\r\n",
        "Content-Type: text/plain\r\n",
        "Content-Length: 13\r\n",
        "\r\n",
        "Hello World!\n"
    );

    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}

impl Server {
    pub fn close(&mut self) -> io::Result<()> {
        self.is_closed.store(true, Ordering::Release);

        if let Some(listener_thread) = self.listener_thread.take() {
            listener_thread.join().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "listener thread panicked",
                )
            })?;
        }

        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.close();
    }
}