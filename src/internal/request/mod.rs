use crate::internal::headers::Headers;
use std::io::{self, Read};

const BUFFER_SIZE: usize = 8;

#[derive(Debug, PartialEq)]
enum ParserState {
    Initialized,
    ParsingHeaders,
    ParsingBody,
    Done,
}

#[derive(Debug)]
pub struct Request {
    pub request_line: Option<RequestLine>,
    pub headers: Headers,
    pub body: Vec<u8>,
    state: ParserState,
}

#[derive(Debug, PartialEq)]
pub struct RequestLine {
    pub http_version: String,
    pub request_target: String,
    pub method: String,
}

impl Request {
    fn new() -> Self {
        Self {
            request_line: None,
            headers: Headers::new(),
            body: Vec::new(),
            state: ParserState::Initialized,
        }
    }

    fn parse(&mut self, data: &[u8]) -> io::Result<usize> {
        let mut total_consumed = 0;

        while self.state != ParserState::Done {
            let consumed = self.parse_single(&data[total_consumed..])?;

            if consumed == 0 {
                break;
            }

            total_consumed += consumed;
        }

        Ok(total_consumed)
    }

    fn parse_single(&mut self, data: &[u8]) -> io::Result<usize> {
        match self.state {
            ParserState::Initialized => {
                let (request_line, consumed) = parse_request_line(data)?;

                if consumed == 0 {
                    return Ok(0);
                }

                self.request_line = Some(request_line);
                self.state = ParserState::ParsingHeaders;

                Ok(consumed)
            }

            ParserState::ParsingHeaders => {
                let (consumed, done) = self.headers.parse(data)?;

                if consumed == 0 {
                    return Ok(0);
                }

                if done {
                    self.state = ParserState::ParsingBody;
                }

                Ok(consumed)
            }

            ParserState::ParsingBody => {
                let Some(content_length_value) = self.headers.get("content-length") else {
                    self.state = ParserState::Done;
                    return Ok(0);
                };

                let content_length = content_length_value.parse::<usize>().map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "invalid content-length")
                })?;

                self.body.extend_from_slice(data);

                if self.body.len() > content_length {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "body larger than content-length",
                    ));
                }

                if self.body.len() == content_length {
                    self.state = ParserState::Done;
                }

                Ok(data.len())
            }

            ParserState::Done => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "parser is already done",
            )),
        }
    }
}

pub fn request_from_reader(mut reader: impl Read) -> io::Result<Request> {
    let mut request = Request::new();

    let mut buffer = vec![0u8; BUFFER_SIZE];
    let mut read_to_index = 0;

    while request.state != ParserState::Done {
        if read_to_index == buffer.len() {
            buffer.resize(buffer.len() * 2, 0);
        }

        let bytes_read = reader.read(&mut buffer[read_to_index..])?;

        if bytes_read == 0 {
            break;
        }

        read_to_index += bytes_read;

        let consumed = request.parse(&buffer[..read_to_index])?;

        if consumed > 0 {
            buffer.copy_within(consumed..read_to_index, 0);
            read_to_index -= consumed;
        }
    }

    if request.state != ParserState::Done {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "incomplete request",
        ));
    }

    Ok(request)
}

fn parse_request_line(data: &[u8]) -> io::Result<(RequestLine, usize)> {
    let Some(pos) = find_crlf(data) else {
        return Ok((
            RequestLine {
                method: String::new(),
                request_target: String::new(),
                http_version: String::new(),
            },
            0,
        ));
    };

    let line = std::str::from_utf8(&data[..pos])
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid utf-8"))?;

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
        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid method"));
    }

    if version != "HTTP/1.1" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported HTTP version",
        ));
    }

    Ok((
        RequestLine {
            method: method.to_string(),
            request_target: request_target.to_string(),
            http_version: "1.1".to_string(),
        },
        pos + 2,
    ))
}

fn find_crlf(data: &[u8]) -> Option<usize> {
    data.windows(2).position(|window| window == b"\r\n")
}

#[cfg(test)]
mod tests;
