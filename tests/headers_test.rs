use miarh::headers::parse_headers;


#[test]
fn reject_if_no_rn() {
    let buf = "GET / HTTP/1.1";
    let r = parse_headers(&buf.as_bytes().to_vec());
    assert_eq!(false, r.is_valid());
}

#[test]
fn reject_if_bad_utf8() {
    let mut valid = "GET / HTTP/1.1\r\nHost: example.com\r\n\r\n"
        .as_bytes().to_vec();
    let r = parse_headers(&valid);
    assert_eq!(true, r.is_valid());

    let bad_utf_code: u8 = 192;
    let mut invalid: Vec<u8> = vec![bad_utf_code];
    invalid.append(&mut valid);
    let r = parse_headers(&invalid);
    assert_eq!(false, r.is_valid());
}
