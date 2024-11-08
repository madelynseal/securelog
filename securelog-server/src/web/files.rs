use actix_web::HttpResponse;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/js"]
struct JSFiles;

#[derive(RustEmbed)]
#[folder = "src/html"]
struct HtmlFiles;

pub fn html_file_response(path: &str) -> HttpResponse {
    match HtmlFiles::get(path) {
        Some(content) => HttpResponse::Ok()
            .content_type("text/html")
            .body(content.data.into_owned()),
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

pub fn js_file_response(path: &str) -> HttpResponse {
    match JSFiles::get(path) {
        Some(content) => HttpResponse::Ok()
            .content_type("application/javascript")
            .body(content.data.into_owned()),
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}
