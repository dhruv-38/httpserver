use super::Headers;

#[test]
fn valid_single_header() {
    let mut headers = Headers::new();

    let data = b"HoSt: localhost:42069\r\n\r\n";

    let (n, done) = headers.parse(data).unwrap();

    assert_eq!(headers.get("host").unwrap(), "localhost:42069");
    assert_eq!(n, 23);
    assert!(!done);
}

#[test]
fn invalid_character_header() {
    let mut headers = Headers::new();

    let data = "H©st: localhost:42069\r\n\r\n".as_bytes();

    let result = headers.parse(data);

    assert!(result.is_err());
}

#[test]
fn duplicate_header_values_are_combined() {
    let mut headers = Headers::new();

    let data1 = b"Set-Person: lane-loves-go\r\n";
    let data2 = b"Set-Person: prime-loves-zig\r\n";

    headers.parse(data1).unwrap();
    headers.parse(data2).unwrap();

    assert_eq!(
        headers.get("set-person").unwrap(),
        "lane-loves-go, prime-loves-zig"
    );
}

#[test]
fn valid_single_header_with_extra_whitespace() {
    let mut headers = Headers::new();

    let data = b"Host:           localhost:42069    \r\n\r\n";

    let (_n, done) = headers.parse(data).unwrap();

    assert_eq!(headers.get("host").unwrap(), "localhost:42069");
    assert!(!done);
}

#[test]
fn valid_two_headers_existing_headers() {
    let mut headers = Headers::new();

    let data = b"Host: localhost:42069\r\nUser-Agent: curl\r\n\r\n";

    let (n, done) = headers.parse(data).unwrap();
    assert_eq!(headers.get("host").unwrap(), "localhost:42069");
    assert!(!done);

    let (n2, done2) = headers.parse(&data[n..]).unwrap();
    assert_eq!(headers.get("user-agent").unwrap(), "curl");
    assert!(!done2);
    assert_eq!(n + n2, 41);
}

#[test]
fn valid_done() {
    let mut headers = Headers::new();

    let data = b"\r\n";

    let (n, done) = headers.parse(data).unwrap();

    assert_eq!(n, 2);
    assert!(done);
}

#[test]
fn invalid_spacing_header() {
    let mut headers = Headers::new();

    let data = b"       Host: localhost:42069\r\n\r\n";

    let result = headers.parse(data);

    assert!(result.is_err());
}