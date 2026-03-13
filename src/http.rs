pub struct Resp {
    pub code: u16,
    pub text: String,
    pub content_type: String,
}

impl Resp {
    pub fn get_resp(&self) -> String {
        format!(
            "HTTP/1.1 {} {}\r\n\
            Content-Length: {}\r\n\
            Content-Type: {}\r\n\
            \r\n{}",
            self.code,
            status_text(self.code),
            self.text.len(),
            self.content_type,
            self.text
        )
    }
}

fn status_text(code: u16) -> &'static str {
    match code {
        200 => "OK",
        301 => "Moved Permanently",
        400 => "Bad Request",
        404 => "Not Found",
        413 => "Payload Too Large",
        500 => "Internal Server Error",
        _ => "Unknown",
    }
}

pub fn text_resp(code: u16, text: String) -> Resp {
    Resp {
        code: code,
        text: text,
        content_type: "text/html".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::text_resp;

    #[test]
    fn formats_status_line_with_reason_phrase() {
        let resp = text_resp(404, "Not found".to_string()).get_resp();
        assert!(resp.starts_with("HTTP/1.1 404 Not Found\r\n"));
    }
}
