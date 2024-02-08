use memchr::memmem::Finder;
use crate::headers::{RequestParser};
use miarh_saras_http::RequestFile;



pub async fn parse_multipart(hp: &mut RequestParser) {
    let ctype = hp.parsed_headers.get("content-type").unwrap();
    let boundary = format!("--{}", ctype.split("boundary=").nth(1).unwrap());
    let finder_rnrn = Finder::new("\r\n\r\n");
    let finder_boundary = Finder::new(&boundary);
    let it = finder_boundary.find_iter(&hp.body);
    let mut prev_pos = 0;
    let mut result_str: String = "".into();
    let mut idx = 0;
    for pos in it {
        let mut field_name = "";
        let mut is_file = false;
        let mut filename: Option<String> = None;
        let mut content_type: String = "".to_string();
        let content: Vec<u8>;
        let row = &hp.body[prev_pos..pos];

        let meta_start_finder = Finder::new("Content-Disposition");
        for meta_start in meta_start_finder.find_iter(&row) {
            let meta = &row[meta_start..];
            let meta_end = finder_rnrn.find(&meta).unwrap();
            let meta = &meta[..meta_end];
            let meta = std::str::from_utf8(meta).unwrap();
            field_name = meta.split(" name=\"").nth(1).unwrap();
            field_name = field_name.split("\"").nth(0).unwrap();
            filename = match meta.split(" filename=\"").nth(1) {
                None => None,
                Some(v) => {
                    match v.split("\"").nth(0) {
                        None => None,
                        Some(v) => Some(v.to_string())
                    }
                }
            };
            is_file = filename.is_some();
            let default_ctype = "text/plain".to_string();
            content_type = match meta.split("Content-Type: ").nth(1) {
                None => default_ctype,
                Some(v) => {
                    match v.split("\"").nth(0) {
                        None => default_ctype,
                        Some(v) => v.to_string()
                    }
                }
            };
        }

        if pos != 0 {
            let val_start = finder_rnrn.find(&row).unwrap();
            let val_end = row.len() - 2;
            content = row[val_start+4..val_end].to_vec();
            let text: Option<String> = match content_type.as_str() {
                "text/plain" => Some(std::str::from_utf8(&content)
                                     .unwrap().to_string()),
                _ => None,
            };

            if is_file == false {
                let comma = if idx > 0 { "," } else { "" };
                if idx == 0 { result_str.push_str("{"); }
                let v = text.unwrap();
                let is_number = v.parse::<f64>().is_ok();
                if is_number {
                  result_str.push_str(&format!(r#"{comma}"{field_name}": {v} "#));
                } else {
                  result_str.push_str(&format!(r#"{comma}"{field_name}": "{v}" "#));
                }
            } else {
              let f = RequestFile { name: filename.clone().unwrap(), content: content.clone() };
              hp.files.insert(field_name.to_string(), f);
            }
            idx += 1;
        }
        prev_pos = pos;
    }
    result_str.push_str("}");
    hp.body_string = result_str;
}
