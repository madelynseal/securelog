use super::{random_string, Result, SqlError, POOL};
use chrono::{DateTime, Utc};

pub async fn client_authenticate(id: &str, token: &str) -> Result<bool> {
    let client = POOL.get().await?;

    let rows = client
        .query("SELECT * FROM clients WHERE id=$1;", &[&id])
        .await?;

    if !rows.is_empty() {
        let row = &rows[0];
        let enabled: bool = row.get("enabled");

        if !enabled {
            return Ok(false);
        }

        let sqltoken: String = row.get("token");

        let valid = bcrypt::verify(token, &sqltoken)?;

        if valid {
            let result = client
                .execute(
                    "UPDATE clients SET lastconnect=$1 WHERE id=$2;",
                    &[&Utc::now(), &id],
                )
                .await?;
            if result < 1 {
                warn!("lastconnect not updated! id={}", id);
            }
        }
        Ok(valid)
    } else {
        info!("Client does not exist {}", id);
        Err(SqlError::ClientNotExist(id.to_string()))
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientAuth {
    id: String,
    token: String,
}
pub async fn client_auth_create(name: &str) -> Result<ClientAuth> {
    if client_name_exists(name).await? {
        return Err(SqlError::ClientNameExists(name.to_string()));
    }
    let token = random_string(32);
    let sqltoken = bcrypt::hash(&token, bcrypt::DEFAULT_COST)?;

    let client = POOL.get().await?;

    let mut id = random_string(32);
    while client_exists(&id).await? {
        id = random_string(32);
    }

    let ts = Utc::now();

    let _result = client
        .execute(
            "INSERT INTO clients 
            (id, token, name, enabled, created, lastconnect) 
            VALUES($1, $2, $3, $4, $5, $6);",
            &[&id, &sqltoken, &name, &true, &ts, &ts],
        )
        .await?;

    let _result2 = client
        .execute(
            "INSERT INTO client_schedule 
                (id, lastrun, manualrun)
                VALUES($1, $2, $3);",
            &[&id, &Utc::now(), &false],
        )
        .await?;

    Ok(ClientAuth { id, token: token })
}

pub async fn delete_client(id: &str) -> Result<bool> {
    let client = POOL.get().await?;

    let result = client
        .execute("DELETE FROM clients WHERE id=$1;", &[&id])
        .await?;

    Ok(result > 0)
}

pub async fn client_exists(id: &str) -> Result<bool> {
    let client = POOL.get().await?;

    let rows = client
        .query("SELECT id FROM clients WHERE id=$1;", &[&id])
        .await?;

    Ok(!rows.is_empty())
}

pub async fn client_name_exists(name: &str) -> Result<bool> {
    let client = POOL.get().await?;

    let rows = client
        .query("SELECT id FROM clients WHERE name=$1 LIMIT 1;", &[&name])
        .await?;

    Ok(!rows.is_empty())
}

pub async fn client_set_enabled(id: &str, enabled: bool) -> Result<()> {
    let client = POOL.get().await?;

    let _result = client
        .execute(
            "UPDATE clients SET enabled=$1 WHERE id=$2;",
            &[&enabled, &id],
        )
        .await?;

    Ok(())
}

pub async fn client_enabled(id: &str) -> Result<bool> {
    let client = POOL.get().await?;

    let rows = client
        .query("SELECT enabled FROM clients WHERE id=$id LIMIT 1;", &[&id])
        .await?;

    if !rows.is_empty() {
        Ok(rows[0].get("enabled"))
    } else {
        Ok(false)
    }
}

#[derive(Debug, Serialize)]
pub struct SLClient {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub created: DateTime<Utc>,
    pub lastconnect: DateTime<Utc>,
}

pub async fn get_clients() -> Result<Vec<SLClient>> {
    let client = POOL.get().await?;

    let mut clients: Vec<SLClient> = Vec::new();

    let rows = client.query("SELECT * FROM clients;", &[]).await?;

    for row in rows {
        let slclient = SLClient {
            id: row.get("id"),
            enabled: row.get("enabled"),
            name: row.get("name"),
            created: row.get("created"),
            lastconnect: row.get("lastconnect"),
        };

        clients.push(slclient);
    }

    Ok(clients)
}

pub async fn get_client_names() -> Result<Vec<String>> {
    let client = POOL.get().await?;

    let rows = client.query("SELECT name FROM clients;", &[]).await?;

    let mut names: Vec<String> = Vec::new();
    for row in rows {
        names.push(row.get("name"));
    }

    Ok(names)
}

#[derive(Debug)]
pub struct ClientLastRun {
    pub lastrun: DateTime<Utc>,
    pub manualrun: bool,
}
pub async fn get_client_last_run(id: &str) -> Result<ClientLastRun> {
    let client = POOL.get().await?;

    let rows = client
        .query("SELECT * FROM client_schedule WHERE id=$1 LIMIT 1;", &[&id])
        .await?;

    if !rows.is_empty() {
        let row = &rows[0];
        let manualrun: Option<bool> = row.get("manualrun");

        Ok(ClientLastRun {
            lastrun: row.get("lastrun"),
            manualrun: manualrun.unwrap_or(false),
        })
    } else {
        Err(SqlError::ClientNotExist(id.to_string()))
    }
}

pub async fn set_client_last_run(id: &str, dt: DateTime<Utc>) -> Result<()> {
    let client = POOL.get().await?;

    let _result = client
        .execute(
            "UPDATE client_schedule SET lastrun=$1, manualrun=$3 WHERE id=$2;",
            &[&dt, &id, &false],
        )
        .await?;

    Ok(())
}

pub async fn set_client_manual_run(id: &str) -> Result<()> {
    let client = POOL.get().await?;

    let _result = client
        .execute(
            "UPDATE client_schedule SET manualrun=$1 WHERE id=$2;",
            &[&true, &id],
        )
        .await?;

    Ok(())
}
