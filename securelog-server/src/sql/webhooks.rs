use super::{random_string, Result, SqlError, POOL};
use chrono::{DateTime, Utc};

pub async fn add_webhook(name: &str, url: &str, username: &str) -> Result<()> {
    let client = POOL.get().await?;

    let _result = client
        .execute(
            "INSERT INTO webhooks (name, url, username) VALUES($1, $2, $3);",
            &[&name, &url, &username],
        )
        .await?;

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct Webhook {
    pub name: String,
    pub url: String,
    pub username: String,
}
impl Webhook {
    pub fn new(name: &str, url: &str, username: &str) -> Webhook {
        Webhook {
            name: name.to_string(),
            url: url.to_string(),
            username: username.to_string(),
        }
    }
    pub fn get_name(&self) -> String {
        self.name.to_owned()
    }
    pub fn get_url(&self) -> String {
        self.url.to_owned()
    }
    pub fn get_username(&self) -> String {
        self.username.to_owned()
    }
}
pub async fn get_webhooks() -> Result<Vec<Webhook>> {
    let client = POOL.get().await?;

    let rows = client.query("SELECT * FROM webhooks;", &[]).await?;

    let mut hooks: Vec<Webhook> = Vec::new();
    for row in rows {
        hooks.push(Webhook::new(
            row.get("name"),
            row.get("url"),
            row.get("username"),
        ));
    }

    Ok(hooks)
}

pub async fn delete_webhook(name: &str) -> Result<bool> {
    let client = POOL.get().await?;

    let result = client
        .execute("DELETE FROM webhooks WHERE name=$1;", &[&name])
        .await?;

    Ok(result > 0)
}
