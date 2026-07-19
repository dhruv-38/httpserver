use super::request_from_reader;
use std::io::{self, Read};

struct ChunkReader {
    data: Vec<u8>,
    num_bytes_per_read: usize,
    pos: usize,
}

impl ChunkReader {
    fn new(data: &str, num_bytes_per_read: usize) -> Self {
        Self {
            data: data.as_bytes().to_vec(),
            num_bytes_per_read,
            pos: 0,
        }
    }
}

impl Read for ChunkReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.pos >= self.data.len() {
            return Ok(0);
        }

        let max_bytes = std::cmp::min(self.num_bytes_per_read, buf.len());

        let end = std::cmp::min(self.pos + max_bytes, self.data.len());

        let chunk = &self.data[self.pos..end];
        let n = chunk.len();

        buf[..n].copy_from_slice(chunk);

        self.pos += n;

        Ok(n)
    }
}

#[test]
fn good_get_request_line_chunk_3() {
    let raw = "GET / HTTP/1.1\r\nHost: localhost:42069\r\n\r\n";

    let reader = ChunkReader::new(raw, 3);

    let r = request_from_reader(reader).unwrap();

    let line = r.request_line.unwrap();

    assert_eq!(line.method, "GET");
    assert_eq!(line.request_target, "/");
    assert_eq!(line.http_version, "1.1");
}

#[test]
fn good_get_request_line_with_path_chunk_1() {
    let raw = "GET /coffee HTTP/1.1\r\nHost: localhost:42069\r\n\r\n";

    let reader = ChunkReader::new(raw, 1);

    let r = request_from_reader(reader).unwrap();

    let line = r.request_line.unwrap();

    assert_eq!(line.method, "GET");
    assert_eq!(line.request_target, "/coffee");
    assert_eq!(line.http_version, "1.1");
}

#[test]
fn invalid_number_of_parts_in_request_line() {
    let raw = "/coffee HTTP/1.1\r\nHost: localhost:42069\r\n\r\n";

    let reader = ChunkReader::new(raw, 8);

    let err = request_from_reader(reader);

    assert!(err.is_err());
}

#[test]
fn invalid_method_request_line() {
    let raw = "get /coffee HTTP/1.1\r\nHost: localhost:42069\r\n\r\n";

    let reader = ChunkReader::new(raw, 8);

    let err = request_from_reader(reader);

    assert!(err.is_err());
}

#[test]
fn invalid_version_in_request_line() {
    let raw = "GET /coffee HTTP/1.0\r\nHost: localhost:42069\r\n\r\n";

    let reader = ChunkReader::new(raw, 8);

    let err = request_from_reader(reader);

    assert!(err.is_err());
}

#[test]
fn standard_headers() {
    let raw = "GET / HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
    let reader = ChunkReader::new(raw, 3);

    let r = request_from_reader(reader).unwrap();

    assert_eq!(r.headers.get("host").unwrap(), "localhost:42069");
    assert_eq!(r.headers.get("user-agent").unwrap(), "curl/7.81.0");
    assert_eq!(r.headers.get("accept").unwrap(), "*/*");
}

#[test]
fn malformed_header() {
    let raw = "GET / HTTP/1.1\r\nHost localhost:42069\r\n\r\n";
    let reader = ChunkReader::new(raw, 3);

    let err = request_from_reader(reader);

    assert!(err.is_err());
}

#[test]
fn empty_headers() {
    let raw = "GET / HTTP/1.1\r\n\r\n";
    let reader = ChunkReader::new(raw, 2);

    let r = request_from_reader(reader).unwrap();

    assert!(r.headers.get("host").is_none());
}

#[test]
fn duplicate_headers() {
    let raw = "GET / HTTP/1.1\r\nSet-Person: lane\r\nSet-Person: prime\r\n\r\n";
    let reader = ChunkReader::new(raw, 4);

    let r = request_from_reader(reader).unwrap();

    assert_eq!(r.headers.get("set-person").unwrap(), "lane, prime");
}

#[test]
fn case_insensitive_headers() {
    let raw = "GET / HTTP/1.1\r\nHoSt: localhost:42069\r\nUSER-Agent: curl\r\n\r\n";
    let reader = ChunkReader::new(raw, 5);

    let r = request_from_reader(reader).unwrap();

    assert_eq!(r.headers.get("host").unwrap(), "localhost:42069");
    assert_eq!(r.headers.get("user-agent").unwrap(), "curl");
}

#[test]
fn missing_end_of_headers() {
    let raw = "GET / HTTP/1.1\r\nHost: localhost:42069\r\n";
    let reader = ChunkReader::new(raw, 8);

    let err = request_from_reader(reader);

    assert!(err.is_err());
}

#[test]
fn standard_body() {
    let raw =
        "POST /submit HTTP/1.1\r\n\
         Host: localhost:42069\r\n\
         Content-Length: 13\r\n\
         \r\n\
         hello world!\n";

    let reader = ChunkReader::new(raw, 3);

    let r = request_from_reader(reader).unwrap();

    assert_eq!(String::from_utf8(r.body).unwrap(), "hello world!\n");
}

#[test]
fn empty_body_zero_content_length() {
    let raw =
        "POST /submit HTTP/1.1\r\n\
         Host: localhost:42069\r\n\
         Content-Length: 0\r\n\
         \r\n";

    let reader = ChunkReader::new(raw, 3);

    let r = request_from_reader(reader).unwrap();

    assert!(r.body.is_empty());
}

#[test]
fn empty_body_without_content_length() {
    let raw =
        "GET / HTTP/1.1\r\n\
         Host: localhost:42069\r\n\
         \r\n";

    let reader = ChunkReader::new(raw, 3);

    let r = request_from_reader(reader).unwrap();

    assert!(r.body.is_empty());
}

#[test]
fn body_shorter_than_content_length() {
    let raw =
        "POST /submit HTTP/1.1\r\n\
         Host: localhost:42069\r\n\
         Content-Length: 20\r\n\
         \r\n\
         partial content";

    let reader = ChunkReader::new(raw, 3);

    let result = request_from_reader(reader);

    assert!(result.is_err());
}

#[test]
fn no_content_length_but_body_exists() {
    let raw =
        "POST /submit HTTP/1.1\r\n\
         Host: localhost:42069\r\n\
         \r\n\
         ignored body";

    let reader = ChunkReader::new(raw, 3);

    let r = request_from_reader(reader).unwrap();

    assert!(r.body.is_empty());
}
