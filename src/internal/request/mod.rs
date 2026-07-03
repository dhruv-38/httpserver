use std::io::{self, Read};

#[derive(Debug, PartialEq)]
pub struct Request {
    pub request_line: RequestLine,
}

#[derive(Debug, PartialEq)]
pub struct RequestLine {
    pub http_version: String,
    pub request_target: String,
    pub method: String,
}

pub fn request_from_reader(mut reader: impl Read) -> io::Result<Request> {
    let mut raw = String::new();
    reader.read_to_string(&mut raw)?;

    let request_line_text = raw
        .split("\r\n")
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing request line"))?;

    let request_line = parse_request_line(request_line_text)?;

    Ok(Request { request_line })
}

fn parse_request_line(line: &str) -> io::Result<RequestLine> {
    let parts: Vec<&str> = line.split(' ').collect();

    if parts.len() != 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid request line parts",
        ));
    }

    let method = parts[0];
    let request_target = parts[1];
    let version = parts[2];

    if !method.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid method",
        ));
    }

    if version != "HTTP/1.1" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported HTTP version",
        ));
    }

    Ok(RequestLine {
        method: method.to_string(),
        request_target: request_target.to_string(),
        http_version: "1.1".to_string(),
    })
}

#[cfg(test)]
mod tests;