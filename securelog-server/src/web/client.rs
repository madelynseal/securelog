use super::{client_logged_in, user_logged_in};
use crate::models::ClientSearchResult;
use crate::sql;
use actix_identity::Identity;
use actix_web::{get, post, web, HttpResponse, Result};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
struct ClientLogin {
    id: String,
    token: String,
}
#[post("/api/client/login")]
async fn api_client_login(
    params: web::Form<ClientLogin>,
    id: Identity,
) -> actix_web::Result<HttpResponse> {
    if let Some(client_id) = client_logged_in(&id) {
        Ok(HttpResponse::Ok().body("Client already logged in"))
    } else if sql::client::client_authenticate(&params.id, &params.token).await? {
        id.remember(format!("client:{}", &params.id));
        Ok(HttpResponse::Ok().body("Logged in successfully"))
    } else {
        Ok(HttpResponse::Unauthorized().body("Login failed"))
    }
}

#[get("/api/client/logout")]
async fn api_client_logout(id: Identity) -> HttpResponse {
    id.forget();
    HttpResponse::Ok().body("Logged")
}

#[derive(Debug, Deserialize)]
struct ClientCreate {
    username: String,
    password: String,
    name: String,
}
#[post("/api/client/create")]
async fn api_client_create(params: web::Form<ClientCreate>) -> Result<HttpResponse> {
    if sql::user::user_login(&params.username, &params.password).await? {
        let client: sql::client::ClientAuth = sql::client::client_auth_create(&params.name).await?;

        Ok(HttpResponse::Ok().body(serde_json::to_string(&client)?))
    } else {
        Ok(HttpResponse::Unauthorized().finish())
    }
}

#[derive(Debug, Deserialize)]
struct ClientSetEnabled {
    pub id: String,
    pub enabled: Option<bool>,
}
#[post("/api/user/client/set_enabled")]
async fn api_client_set_enabled(
    id: Identity,
    params: web::Form<ClientSetEnabled>,
) -> actix_web::Result<HttpResponse> {
    if let Some(_username) = user_logged_in(&id) {
        let enabled = if let Some(e) = &params.enabled {
            *e
        } else {
            false
        };
        sql::client::client_set_enabled(&params.id, enabled).await?;

        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::Unauthorized().finish())
    }
}

#[get("/api/client/get_searches")]
async fn api_client_get_searches(id: Identity) -> actix_web::Result<HttpResponse> {
    if let Some(_username) = client_logged_in(&id) {
        let searches = sql::get_searches().await?;

        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .json(&searches))
    } else {
        Ok(HttpResponse::Unauthorized().body("Unauthorized"))
    }
}

#[derive(Debug, Deserialize)]
struct ClientSendSearchResults {
    results: String,
}
#[post("/api/client/send_search_results")]
async fn api_client_send_search_results(
    params: web::Form<ClientSendSearchResults>,
    id: Identity,
) -> actix_web::Result<HttpResponse> {
    if let Some(client_id) = client_logged_in(&id) {
        let results: Vec<ClientSearchResult> = serde_json::from_str(&params.results)?;
        for result in &results {
            sql::insert_client_search_result(&client_id, result).await?;
        }

        let message = format!("New scan results for client {} received", client_id);
        crate::webhooks::send_message(&message).await?;

        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::Unauthorized().body("Unauthorized"))
    }
}

#[get("/api/client/notify_running")]
async fn api_client_notify_running(id: Identity) -> actix_web::Result<HttpResponse> {
    if let Some(client_id) = client_logged_in(&id) {
        sql::client::set_client_last_run(&client_id, chrono::Utc::now()).await?;

        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::Unauthorized().body("Unauthorized"))
    }
}

#[get("/api/client/should_run")]
async fn api_client_should_run(id: Identity) -> actix_web::Result<HttpResponse> {
    if let Some(client_id) = client_logged_in(&id) {
        let schedule = sql::get_scan_schedule().await?;
        let lastrun = sql::client::get_client_last_run(&client_id).await?;

        if lastrun.manualrun {
            let data = json!({
                "should_run": "true"
            });
            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(serde_json::to_string(&data)?))
        } else {
            let nextrun =
                lastrun.lastrun + chrono::Duration::from_std(schedule.get_interval()).unwrap();

            // should_run should be false every time if in manual mode
            let should_run = nextrun < Utc::now() && !schedule.is_manual();

            let data = json!({ "should_run": should_run }).to_string();

            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(data))
        }
    } else {
        Ok(HttpResponse::Unauthorized().body("Unauthorized"))
    }
}
