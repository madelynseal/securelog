use crate::conf;
use crate::models::{SearchResult, SearchType};
use crate::{constants, sql};
use actix_identity::{Identity, IdentityMiddleware};
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, post, web, App, HttpResponse, HttpServer, Responder};
use rustls::{Certificate, PrivateKey, ServerConfig};
use std::fs::File;
use std::io::BufReader;

mod client;
mod files;
mod html;
mod user;

pub async fn start() -> std::io::Result<()> {
    let secret_key = Key::generate();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            // Install identity framework
            .wrap(IdentityMiddleware::default())
            .wrap(SessionMiddleware::new(CookieSessionStore::default(), secret_key.clone()))
            .wrap(actix_web::middleware::DefaultHeaders::new()
                .add(
                    ("Content-Security-Policy",
                    "default-src 'self'; script-src 'self' cdn.jsdelivr.net; style-src 'self' cdn.jsdelivr.net;")
                )
                .add(
                    ("X-Frame-Options", "DENY")
                )
            )
            .service(user::api_user_login)
            .service(user::api_user_logout)
            .service(user::api_user_username)
            .service(user::api_user_insert_search)
            .service(user::api_user_delete_search)
            .service(user::api_user_get_search_results)
            .service(user::api_user_set_schedule)
            .service(user::api_user_webhooks_add)
            .service(user::api_user_webhooks_fetch)
            .service(user::api_user_webhooks_delete)
            .service(user::api_user_get_searches)
            .service(user::api_user_client_delete)
            .service(user::api_fetch_clients)
            .service(client::api_client_login)
            .service(client::api_client_logout)
            .service(client::api_client_create)
            .service(client::api_client_set_enabled)
            .service(client::api_client_get_searches)
            .service(client::api_client_send_search_results)
            .service(client::api_client_should_run)
            .service(client::api_client_notify_running)
            .service(html::login)
            .service(html::index)
            .service(html::clients)
            .service(html::search_result_form)
            .service(html::search_results)
            .service(html::js_file)
            .service(html::schedule)
            .service(html::searches)
            .service(html::webhooks)
    });

    let listen_address: String = conf::get_server_listen().unwrap();
    info!("will listen on {}", listen_address);

    if !conf::get_use_https().unwrap_or(false) {
        server.bind(listen_address)?.run().await?;
    } else {
        let cert_file: String = conf::get_server_cert().unwrap();
        let key_file: String = conf::get_server_cert_key().unwrap();

        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth();
        let cert_file = &mut BufReader::new(File::open(cert_file).unwrap());
        let key_file = &mut BufReader::new(File::open(key_file).unwrap());
        let cert_chain = rustls_pemfile::certs(cert_file)
            .unwrap()
            .into_iter()
            .map(Certificate)
            .collect();
        let mut keys: Vec<PrivateKey> = rustls_pemfile::pkcs8_private_keys(key_file)
            .unwrap()
            .into_iter()
            .map(PrivateKey)
            .collect();
        if keys.is_empty() {
            eprintln!("Could not locate PKCS 8 private keys.");
            std::process::exit(1);
        }
        let config = config.with_single_cert(cert_chain, keys.remove(0)).unwrap();
        server.bind_rustls(listen_address, config)?.run().await?;
    }

    Ok(())
}

fn client_logged_in(user: Option<Identity>) -> Option<String> {
    debug!("client_logged_in: {}", user.is_some());
    if let Some(user) = user {
        user.id()
            .unwrap()
            .strip_prefix("client:")
            .map(|client| client.to_string())
    } else {
        None
    }
}

fn user_logged_in(user: Option<Identity>) -> Option<String> {
    debug!("user_logged_in: {}", user.is_some());
    if let Some(user) = user {
        user.id()
            .unwrap()
            .strip_prefix("user:")
            .map(|username| username.to_string())
    } else {
        None
    }
}
