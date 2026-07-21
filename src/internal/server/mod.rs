use crate::internal::request::{Request, request_from_reader};
use crate::internal::response::{StatusCode, Writer as ResponseWriter, get_default_headers};
use std::io::{self, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub type Handler = for<'a> fn(&mut ResponseWriter<'a>, &Request) -> io::Result<()>;

pub struct Server {
    is_closed: Arc<AtomicBool>,
    listener_thread: Option<JoinHandle<()>>,
}

pub fn serve(port: u16, handler: Handler) -> io::Result<Server> {
    let listener = TcpListener::bind(("0.0.0.0", port))?;

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
    let request = match request_from_reader(&mut stream) {
        Ok(request) => request,

        Err(error) => {
            let body = format!("Bad Request: {error}\n");

            {
                let mut writer = ResponseWriter::new(&mut stream);

                writer.write_status_line(StatusCode::BAD_REQUEST)?;

                let headers = get_default_headers(body.len());
                writer.write_headers(headers)?;

                writer.write_body(body.as_bytes())?;
            }

            stream.flush()?;
            return Ok(());
        }
    };

    {
        let mut writer = ResponseWriter::new(&mut stream);
        handler(&mut writer, &request)?;
    }

    stream.flush()?;

    Ok(())
}

impl Server {
    pub fn close(&mut self) -> io::Result<()> {
        self.is_closed.store(true, Ordering::Release);

        if let Some(listener_thread) = self.listener_thread.take() {
            listener_thread
                .join()
                .map_err(|_| io::Error::other("listener thread panicked"))?;
        }

        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
