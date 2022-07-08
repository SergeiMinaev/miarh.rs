use miarh::headers::process_headers;


const CARRIAGE_RETURN: u8 = 13;
const LINE_FEED: u8 = 10;


#[test]
fn reject_if_no_rn() {
    let buf = "GET / HTTP/1.1";
    assert_eq!(false, process_headers(&buf.as_bytes().to_vec()));
}

#[test]
fn reject_if_bad_utf8() {
    let valid: Vec<u8> = vec![102, 111, 111, CARRIAGE_RETURN, LINE_FEED];
    assert_eq!(true, process_headers(&valid));
    let invalid: Vec<u8> = vec![192, 102, 111, 111, CARRIAGE_RETURN, LINE_FEED];
    assert_eq!(false, process_headers(&invalid));
}
