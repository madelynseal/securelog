use super::user_logged_in;
use actix_identity::Identity;
use actix_web::{get, web, HttpResponse, Responder};

#[get("/")]
pub async fn index(id: Identity) -> impl Responder {
    if let Some(_username) = user_logged_in(&id) {
        super::files::html_file_response("index.html")
    } else {
        HttpResponse::Found()
            .insert_header(("location", "/login?redirect=/"))
            .finish()
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginHtml {
    pub redirect: Option<String>,
}
#[get("/login")]
pub async fn login(id: Identity, params: web::Query<LoginHtml>) -> impl Responder {
    if let Some(_username) = user_logged_in(&id) {
        let redirect = if let Some(redirect) = &params.redirect {
            if redirect.starts_with("/") {
                redirect
            } else {
                "/"
            }
        } else {
            "/"
        };
        HttpResponse::Ok()
            .insert_header(("location", redirect))
            .finish()
    } else {
        super::files::html_file_response("login.html")
    }
}

#[get("/clients")]
pub async fn clients(id: Identity) -> HttpResponse {
    if let Some(_username) = user_logged_in(&id) {
        super::files::html_file_response("clients.html")
    } else {
        HttpResponse::Found()
            .insert_header(("location", "/login?redirect=/clients"))
            .finish()
    }
}

#[get("/search_result_form")]
pub async fn search_result_form(id: Identity) -> HttpResponse {
    if let Some(_username) = user_logged_in(&id) {
        super::files::html_file_response("search_result_form.html")
    } else {
        HttpResponse::Found()
            .insert_header(("location", "/login?redirect=/search_result_form"))
            .finish()
    }
}

#[get("/search_results")]
pub async fn search_results(id: Identity) -> HttpResponse {
    if let Some(_username) = user_logged_in(&id) {
        super::files::html_file_response("search_results.html")
    } else {
        HttpResponse::Found()
            .insert_header(("location", "/login?redirect=/"))
            .finish()
    }
}

#[get("/schedule")]
pub async fn schedule(id: Identity) -> HttpResponse {
    if let Some(_username) = user_logged_in(&id) {
        super::files::html_file_response("schedule.html")
    } else {
        HttpResponse::Found()
            .insert_header(("location", "/login?redirect=/sechedule"))
            .finish()
    }
}

#[get("/searches")]
pub async fn searches(id: Identity) -> HttpResponse {
    if let Some(_username) = user_logged_in(&id) {
        super::files::html_file_response("searches.html")
    } else {
        HttpResponse::Found()
            .insert_header(("location", "/login?redirect=/searches"))
            .finish()
    }
}

#[get("/webhooks")]
pub async fn webhooks(id: Identity) -> HttpResponse {
    if let Some(_username) = user_logged_in(&id) {
        super::files::html_file_response("webhooks.html")
    } else {
        HttpResponse::Found()
            .insert_header(("location", "/login?redirect=/webhooks"))
            .finish()
    }
}

#[get("/js/{path}")]
pub async fn js_file(path: web::Path<String>, id: Identity) -> HttpResponse {
    let path = path.into_inner();
    if path == String::from("login.js") {
        return super::files::js_file_response(&path);
    }
    if let Some(_username) = user_logged_in(&id) {
        super::files::js_file_response(&path)
    } else {
        HttpResponse::Unauthorized().body("Unauthorized")
    }
}
