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

pub fn write_status_line(writer: &mut impl Write, status_code: StatusCode) -> io::Result<()> {
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

pub fn write_headers(writer: &mut impl Write, headers: &Headers) -> io::Result<()> {
    for (key, value) in headers.iter() {
        let key = canonical_header_name(key);

        write!(writer, "{}: {}\r\n", key, value)?;
    }

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
