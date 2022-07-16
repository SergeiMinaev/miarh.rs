// Lists of (almost) all known mime types:
// http://www.iana.org/assignments/media-types/media-types.xhtml
// https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Complete_list_of_MIME_types
//
// Rase uses only fairly popular file extensions.

use std::path::Path;


pub fn get_mimetype (fname: &String) -> Option<String> {
    let r = Path::new(&fname).extension();
    if !r.is_some() {
        return None;
    }
    let m = match r.unwrap().to_str().unwrap().to_ascii_lowercase().as_str() {
        // application
        "7z" => "application/x-7z-compressed",
        "bin" => "application/octet-stream",
        "bz" => "application/x-bzip",
        "bz2" => "application/x-bzip2",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "eot" => "application/vnd.ms-fontobject",
        "gz" => "application/gzip",
        "gzip" => "application/gzip",
        "json" => "application/json",
        "odp" => "application/vnd.oasis.opendocument.presentation",
        "ods" => "application/vnd.oasis.opendocument.spreadsheet",
        "odt" => "application/vnd.oasis.opendocument.text",
        "pdf" => "application/pdf",
        "ppt" => "application/vnd.ms-powerpoint",
        "rar" => "application/x-rar-compressed",
        "sh" => "application/x-sh",
        "sql" => "application/sql",
        "tar" => "application/x-tar",
        "xls" => "application/vnd.ms-excel",
        "zip" => "application/zip",
        // audio
        "aac" => "audio/aac",
        "oga" => "audio/ogg",
        "ogg" => "audio/ogg",
        "opus" => "audio/opus",
        "mp3" => "audio/mpeg",
        // image
        "gif" => "image/gif",
        "ico" => "image/vnd.microsoft.icon",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "jxl" => "image/jxl",
        // font
        "otf" => "font/otf",
        "ttf" => "font/ttf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        // text
        "css" => "text/css",
        "csv" => "text/csv",
        "htm" => "text/html",
        "html" => "text/html",
        "js" => "text/javascript",
        "mjs" => "text/javascript",
        "rtf" => "text/rtf",
        "txt" => "text/plain",
        "xml" => "text/xml",
        // video
        "3gp" => "video/3gpp",
        "3gpp" => "video/3gpp",
        "avi" => "video/x-msvideo",
        "H261" => "video/H261",
        "H263" => "video/H263",
        "H264" => "video/H264",
        "H265" => "video/H265",
        "mp4" => "video/mp4",
        "mpeg" => "video/mpeg",
        "mpg" => "video/mpeg",
        "ogv" => "video/ogg",
        "vp8" => "video/VP8",
        "webm" => "video/webm",
        _ => return None,
    };
    return Some(m.to_string());
}
