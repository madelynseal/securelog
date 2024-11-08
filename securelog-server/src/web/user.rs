use super::{client_logged_in, user_logged_in};
use crate::models::{SearchResult, SearchType};
use crate::sql;
use actix_identity::Identity;
use actix_web::{get, post, web, HttpResponse, Responder, Result};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
struct AuthLogin {
    username: String,
    password: String,
}
#[derive(Debug, Deserialize)]
struct AuthLoginQuery {
    redirect: Option<String>,
}
#[post("/api/user/login")]
async fn api_user_login(
    params: web::Form<AuthLogin>,
    query: web::Query<AuthLoginQuery>,
    id: Identity,
) -> actix_web::Result<HttpResponse> {
    if sql::user::user_login(&params.username, &params.password).await? {
        id.remember(format!("user:{}", &params.username));

        let redirect = if let Some(redirect) = &query.redirect {
            if redirect.starts_with('/') {
                redirect
            } else {
                "/"
            }
        } else {
            "/"
        };

        Ok(HttpResponse::Found()
            .insert_header(("location", redirect))
            .finish())
    } else {
        Ok(HttpResponse::Found()
            .insert_header(("location", "/login"))
            .finish())
    }
}

#[post("/api/user/logout")]
async fn api_user_logout(id: Identity) -> impl Responder {
    id.forget();

    HttpResponse::Found()
        .insert_header(("location", "/"))
        .finish()
}

#[get("/api/user/username")]
async fn api_user_username(id: Identity) -> impl Responder {
    if let Some(username) = user_logged_in(&id) {
        HttpResponse::Ok().body(username)
    } else {
        HttpResponse::Ok().body("Null")
    }
}

#[get("/api/user/client/fetch_all")]
async fn api_fetch_clients(id: Identity) -> actix_web::Result<HttpResponse> {
    if let Some(_username) = user_logged_in(&id) {
        let clients = sql::client::get_clients().await?;
        let json = serde_json::to_string(&clients)?;

        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(json))
    } else {
        Ok(HttpResponse::Unauthorized().finish())
    }
}

#[derive(Debug, Deserialize)]
struct UserInsertSearch {
    name: String,
    stype: SearchType,
    search: String,
    locations: String,
}

#[post("/api/user/create_search")]
async fn api_user_insert_search(
    id: Identity,
    params: web::Form<UserInsertSearch>,
) -> actix_web::Result<HttpResponse> {
    if let Some(_username) = user_logged_in(&id) {
        let mut locations: Vec<String> = Vec::new();
        for line in params.locations.lines() {
            locations.push(line.to_string());
        }

        let _id =
            sql::insert_search(&params.name, &params.stype, &params.search, &locations).await?;

        Ok(HttpResponse::Found()
            .insert_header(("location", "/searches"))
            .finish())
    } else {
        Ok(HttpResponse::Found()
            .insert_header(("location", "/login"))
            .finish())
    }
}
#[derive(Debug, Deserialize)]
struct UserDeleteSearch {
    id: i32,
}
#[post("/api/user/delete_search")]
async fn api_user_delete_search(
    id: Identity,
    params: web::Form<UserDeleteSearch>,
) -> actix_web::Result<HttpResponse> {
    if let Some(_username) = user_logged_in(&id) {
        sql::delete_search(params.id).await?;

        Ok(HttpResponse::Found()
            .insert_header(("location", "/searches"))
            .finish())
    } else {
        Ok(HttpResponse::Found()
            .insert_header(("location", "/login"))
            .finish())
    }
}

#[derive(Debug, Deserialize)]
struct UserGetSearchResults {
    client: Option<String>,
    before: Option<DateTime<Utc>>,
    after: Option<DateTime<Utc>>,
}
#[get("/api/user/get_search_results")]
async fn api_user_get_search_results(
    id: Identity,
    params: web::Query<UserGetSearchResults>,
) -> actix_web::Result<HttpResponse> {
    if let Some(_username) = user_logged_in(&id) {
        let results =
            sql::get_search_results(params.client.clone(), params.before, params.after).await?;

        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .json(&results))
    } else {
        Ok(HttpResponse::Found()
            .insert_header(("location", "/login"))
            .finish())
    }
}

#[derive(Debug, Deserialize)]
struct UserSetSchedule {
    schedule: u64,
    manual: Option<bool>,
}
#[post("/api/user/set_schedule")]
async fn api_user_set_schedule(
    id: Identity,
    params: web::Form<UserSetSchedule>,
) -> actix_web::Result<HttpResponse> {
    if let Some(_username) = user_logged_in(&id) {
        sql::set_search_schedule_minutes(params.schedule as i32, params.manual.unwrap_or(false))
            .await?;

        Ok(HttpResponse::Found()
            .insert_header(("location", "/"))
            .finish())
    } else {
        Ok(HttpResponse::Found()
            .insert_header(("location", "/login"))
            .finish())
    }
}

#[get("/api/user/webhooks/fetch")]
async fn api_user_webhooks_fetch(id: Identity) -> actix_web::Result<HttpResponse> {
    if let Some(_username) = user_logged_in(&id) {
        let webhooks = sql::webhooks::get_webhooks().await?;

        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(serde_json::to_string(&webhooks)?))
    } else {
        Ok(HttpResponse::Found()
            .insert_header(("location", "/login"))
            .finish())
    }
}

#[derive(Debug, Deserialize)]
struct WebhookAdd {
    pub name: String,
    pub url: String,
    pub username: String,
}
#[post("/api/user/webhooks/add")]
async fn api_user_webhooks_add(
    id: Identity,
    params: web::Form<WebhookAdd>,
) -> actix_web::Result<HttpResponse> {
    if let Some(_username) = user_logged_in(&id) {
        sql::webhooks::add_webhook(&params.name, &params.url, &params.username).await?;

        Ok(HttpResponse::Found()
            .insert_header(("location", "/"))
            .finish())
    } else {
        Ok(HttpResponse::Found()
            .insert_header(("location", "/login"))
            .finish())
    }
}

#[derive(Debug, Deserialize)]
struct WebhookDelete {
    name: String,
}
#[post("/api/user/webhooks/delete")]
async fn api_user_webhooks_delete(
    id: Identity,
    params: web::Form<WebhookDelete>,
) -> actix_web::Result<HttpResponse> {
    if let Some(_username) = user_logged_in(&id) {
        sql::webhooks::delete_webhook(&params.name).await?;

        Ok(HttpResponse::Found()
            .insert_header(("location", "/"))
            .finish())
    } else {
        Ok(HttpResponse::Found()
            .insert_header(("location", "/login"))
            .finish())
    }
}

#[get("/api/user/get_searches")]
async fn api_user_get_searches(id: Identity) -> actix_web::Result<HttpResponse> {
    if let Some(_username) = user_logged_in(&id) {
        let searches = sql::get_searches().await?;

        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(serde_json::to_string(&searches).unwrap()))
    } else {
        Ok(HttpResponse::Found()
            .insert_header(("location", "/login"))
            .finish())
    }
}

#[derive(Debug, Deserialize)]
struct ClientDelete {
    id: String,
}
#[post("/api/user/client/delete")]
async fn api_user_client_delete(
    id: Identity,
    params: web::Form<ClientDelete>,
) -> actix_web::Result<HttpResponse> {
    if let Some(_username) = user_logged_in(&id) {
        if sql::client::delete_client(&params.id).await? {
            Ok(HttpResponse::Found()
                .insert_header(("location", "/clients"))
                .finish())
        } else {
            Ok(HttpResponse::InternalServerError().body("Failed to delete client"))
        }
    } else {
        Ok(HttpResponse::Found()
            .insert_header(("location", "/login"))
            .finish())
    }
}
