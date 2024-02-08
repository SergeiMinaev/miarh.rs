pub struct Resp {
    pub code: u16,
    pub text: String,
    pub content_type: String,
}

impl Resp {
    pub fn get_resp(&self) -> String {
        format!(
            "HTTP/1.1 {}\r\n\
            Content-Length: {}\r\n\
            Content-Type: {}\r\n\
            \r\n{}",
            self.code, self.text.len(), self.content_type, self.text
        )
    }
}

pub fn text_resp(code: u16, text: String) -> Resp {
    Resp {
        code: code,
        text: text,
        content_type: "text/html".to_string(),
    }
}

