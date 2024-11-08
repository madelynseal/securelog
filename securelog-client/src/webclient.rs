use crate::{
    conf,
    models::{Search, SearchResult},
};
use reqwest::blocking::Client;
use reqwest::StatusCode;

#[derive(Debug, Error)]
pub enum WebError {
    #[error("WebError(Reqwest({0}))")]
    Reqwest(#[from] reqwest::Error),

    #[error("WebError(Config({0}))")]
    Config(#[from] config::ConfigError),

    #[error("WebError(Json({0}))")]
    Json(#[from] serde_json::Error),
}
pub type Result<T> = std::result::Result<T, WebError>;

lazy_static! {
    // reqwest uses an internal connection pool
    // so we should reuse the client each time
    static ref CLIENT: Client = Client::builder()
        .cookie_store(true).build().unwrap();
}

pub fn login() -> Result<bool> {
    let server = conf::get_server()?;
    let id = conf::get_id()?;
    let token = conf::get_token()?;

    let params = json!({
        "id": id,
        "token": token
    });

    let url = format!("{}/api/client/login", server);

    let result = CLIENT.post(&url).form(&params).send()?;
    let status = result.status();
    let text = result.text()?;

    match status {
        StatusCode::OK => Ok(true),
        StatusCode::UNAUTHORIZED => Ok(false),
        _ => {
            warn!("login: Unexpected status {}, text={}", status, text);
            Ok(false)
        }
    }
}

pub fn logout() -> Result<bool> {
    let server = conf::get_server()?;

    let url = format!("{}/api/client/logout", server);

    let result = CLIENT.get(&url).send()?;
    let status = result.status();
    let text = result.text()?;

    match status {
        StatusCode::OK => Ok(true),
        _ => {
            warn!(
                "Unexpected error code while logging out: {}, text={}",
                status, text
            );
            Ok(false)
        }
    }
}

pub fn get_searches() -> Result<Vec<Search>> {
    let server = conf::get_server()?;

    let url = format!("{}/api/client/get_searches", server);

    let result = CLIENT.get(&url).send()?;
    let status = result.status();
    let text = result.text()?;

    match status {
        StatusCode::OK => {
            let json: Vec<Search> = serde_json::from_str(&text)?;

            Ok(json)
        }
        StatusCode::UNAUTHORIZED => {
            warn!("Not logged in, text={}", text);
            Ok(Vec::new())
        }
        _ => {
            warn!("Unexpected status code: {}, text={}", status, text);
            Ok(Vec::new())
        }
    }
}

pub fn send_search_results(result: &[SearchResult]) -> Result<bool> {
    let server = conf::get_server()?;

    let params = json!({ "results": serde_json::to_string(&result)? });

    let url = format!("{}/api/client/send_search_results", server);

    let result = CLIENT.post(&url).form(&params).send()?;

    if result.status() != StatusCode::OK {
        warn!(
            "send_search_results: unexpected server error: {}, text={}",
            result.status(),
            result.text()?
        );
        Ok(false)
    } else {
        Ok(true)
    }
}

#[derive(Debug, Deserialize)]
pub struct ClientAuth {
    pub id: String,
    pub token: String,
}
pub fn create_client_token(
    server: &str,
    name: &str,
    username: &str,
    password: &str,
) -> Result<ClientAuth> {
    let params = json!({
        "name": name,
        "username": username,
        "password": password,
    });

    let url = format!("{}/api/client/create", server);

    let result = CLIENT.post(&url).form(&params).send()?;
    let status = result.status();
    let text = result.text()?;

    match status {
        StatusCode::OK => {
            let resp: ClientAuth = serde_json::from_str(&text)?;

            Ok(resp)
        }
        StatusCode::UNAUTHORIZED => {
            panic!("failed to login! {}", text);
        }
        StatusCode::INTERNAL_SERVER_ERROR => {
            panic!("internal server error: {}", text);
        }
        _ => {
            panic!("Unexpected status code {}, text={}", status, text);
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ClientShouldRun {
    pub should_run: bool,
}
pub fn get_should_run() -> Result<bool> {
    let server = conf::get_server()?;

    let url = format!("{}/api/client/should_run", server);

    let result = CLIENT.get(&url).send()?;
    let status = result.status();
    let text = result.text()?;

    match status {
        StatusCode::INTERNAL_SERVER_ERROR => {
            warn!("server error: {}", text);
            Ok(false)
        }
        StatusCode::UNAUTHORIZED => {
            warn!("no longer logged in: {}", text);
            Ok(false)
        }
        StatusCode::OK => {
            let resp: ClientShouldRun = serde_json::from_str(&text)?;

            Ok(resp.should_run)
        }
        _ => {
            warn!("Unexpected error code {}, text={}", status, text);
            Ok(false)
        }
    }
}

pub fn notify_running() -> Result<()> {
    let server = conf::get_server()?;

    let url = format!("{}/api/client/notify_running", server);

    let result = CLIENT.get(&url).send()?;

    if result.status() != StatusCode::OK {
        warn!(
            "notify_running: Unexpected status code: {}",
            result.status()
        );
    }

    Ok(())
}
