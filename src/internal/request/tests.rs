use super::request_from_reader;
use std::io::Cursor;

#[test]
fn good_get_request_line() {
    let raw = "GET / HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";

    let r = request_from_reader(Cursor::new(raw)).unwrap();

    assert_eq!(r.request_line.method, "GET");
    assert_eq!(r.request_line.request_target, "/");
    assert_eq!(r.request_line.http_version, "1.1");
}

#[test]
fn good_get_request_line_with_path() {
    let raw = "GET /coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";

    let r = request_from_reader(Cursor::new(raw)).unwrap();

    assert_eq!(r.request_line.method, "GET");
    assert_eq!(r.request_line.request_target, "/coffee");
    assert_eq!(r.request_line.http_version, "1.1");
}

#[test]
fn good_post_request_with_path() {
    let raw = "POST /coffee HTTP/1.1\r\nHost: localhost:42069\r\nContent-Length: 22\r\n\r\n{\"flavor\":\"dark mode\"}";

    let r = request_from_reader(Cursor::new(raw)).unwrap();

    assert_eq!(r.request_line.method, "POST");
    assert_eq!(r.request_line.request_target, "/coffee");
    assert_eq!(r.request_line.http_version, "1.1");
}

#[test]
fn invalid_number_of_parts_in_request_line() {
    let raw = "/coffee HTTP/1.1\r\nHost: localhost:42069\r\n\r\n";

    let err = request_from_reader(Cursor::new(raw));

    assert!(err.is_err());
}

#[test]
fn invalid_method_request_line() {
    let raw = "get /coffee HTTP/1.1\r\nHost: localhost:42069\r\n\r\n";

    let err = request_from_reader(Cursor::new(raw));

    assert!(err.is_err());
}

#[test]
fn invalid_version_in_request_line() {
    let raw = "GET /coffee HTTP/1.0\r\nHost: localhost:42069\r\n\r\n";

    let err = request_from_reader(Cursor::new(raw));

    assert!(err.is_err());
}