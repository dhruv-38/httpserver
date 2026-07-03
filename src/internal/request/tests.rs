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

        let end = std::cmp::min(
            self.pos + self.num_bytes_per_read,
            self.data.len(),
        );

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