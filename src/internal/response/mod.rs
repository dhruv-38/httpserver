use std::io::{self, Write};

use crate::internal::headers::Headers;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusCode(u16);

impl StatusCode {
    pub const OK: Self = Self(200);
    pub const BAD_REQUEST: Self = Self(400);
    pub const INTERNAL_SERVER_ERROR: Self = Self(500);

    pub const fn new(code: u16) -> Self {
        Self(code)
    }

    pub const fn as_u16(self) -> u16 {
        self.0
    }
}

pub fn write_status_line<W: Write + ?Sized>(
    writer: &mut W,
    status_code: StatusCode,
) -> io::Result<()> {
    let reason_phrase = match status_code.as_u16() {
        200 => "OK",
        400 => "Bad Request",
        500 => "Internal Server Error",
        _ => "",
    };

    write!(
        writer,
        "HTTP/1.1 {} {}\r\n",
        status_code.as_u16(),
        reason_phrase
    )
}

pub fn get_default_headers(content_len: usize) -> Headers {
    let mut headers = Headers::new();

    headers.set("Content-Length", &content_len.to_string());
    headers.set("Connection", "close");
    headers.set("Content-Type", "text/plain");

    headers
}

pub fn write_headers<W: Write + ?Sized>(writer: &mut W, headers: &Headers) -> io::Result<()> {
    for (key, value) in headers.iter() {
        let key = canonical_header_name(key);

        write!(writer, "{}: {}\r\n", key, value)?;
    }

    // Empty line marks the end of the headers.
    writer.write_all(b"\r\n")
}

fn canonical_header_name(key: &str) -> String {
    let mut result = String::with_capacity(key.len());
    let mut uppercase_next = true;

    for character in key.chars() {
        if uppercase_next {
            result.push(character.to_ascii_uppercase());
            uppercase_next = false;
        } else {
            result.push(character.to_ascii_lowercase());
        }

        if character == '-' {
            uppercase_next = true;
        }
    }

    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WriterState {
    Initialized,
    StatusLineWritten,
    HeadersWritten,
    BodyWritten,
    ChunkedBodyWritten,
    Done,
}

pub struct Writer<'a> {
    inner: &'a mut dyn Write,
    state: WriterState,
}

impl<'a> Writer<'a> {
    pub fn new(inner: &'a mut dyn Write) -> Self {
        Self {
            inner,
            state: WriterState::Initialized,
        }
    }

    pub fn write_status_line(&mut self, status_code: StatusCode) -> io::Result<()> {
        if self.state != WriterState::Initialized {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "status line must be written first",
            ));
        }

        write_status_line(self.inner, status_code)?;
        self.state = WriterState::StatusLineWritten;

        Ok(())
    }

    pub fn write_headers(&mut self, headers: Headers) -> io::Result<()> {
        if self.state != WriterState::StatusLineWritten {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "headers must be written after the status line",
            ));
        }

        write_headers(self.inner, &headers)?;
        self.state = WriterState::HeadersWritten;

        Ok(())
    }

    pub fn write_body(&mut self, body: &[u8]) -> io::Result<usize> {
        match self.state {
            WriterState::HeadersWritten | WriterState::BodyWritten => {}
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "body must be written after the headers",
                ));
            }
        }

        self.inner.write_all(body)?;
        self.state = WriterState::BodyWritten;

        Ok(body.len())
    }

    pub fn write_chunked_body(&mut self, body: &[u8]) -> io::Result<usize> {
        match self.state {
            WriterState::HeadersWritten | WriterState::ChunkedBodyWritten => {}

            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "chunked body must be written after the headers",
                ));
            }
        }

        if body.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "use write_chunked_body_done for the final empty chunk",
            ));
        }

        // Write the body length in hexadecimal followed by CRLF.
        write!(self.inner, "{:x}\r\n", body.len())?;

        // Write the actual chunk data.
        self.inner.write_all(body)?;

        // Each chunk ends with CRLF.
        self.inner.write_all(b"\r\n")?;

        self.state = WriterState::ChunkedBodyWritten;

        Ok(body.len())
    }

    pub fn write_chunked_body_done(&mut self) -> io::Result<usize> {
        match self.state {
            WriterState::HeadersWritten | WriterState::ChunkedBodyWritten => {}

            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "chunked body must be finished after the headers",
                ));
            }
        }

        self.inner.write_all(b"0\r\n\r\n")?;
        self.state = WriterState::Done;

        Ok(5)
    }
}
