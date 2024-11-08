use super::{Result, SqlError, POOL};
use chrono::Utc;

pub async fn user_login(username: &str, passwd: &str) -> Result<bool> {
    let client = POOL.get().await?;

    let rows = client
        .query(
            "SELECT * FROM auth WHERE username=$1 LIMIT 1;",
            &[&username],
        )
        .await?;

    if !rows.is_empty() {
        let enabled: bool = rows[0].get("enabled");
        if !enabled {
            warn!("login for {} failed: account not enabled", username);
            return Err(SqlError::UserDisabled);
        }

        let sqlpasswd: String = rows[0].get("passwd");

        if bcrypt::verify(passwd, &sqlpasswd)? {
            warn!("successful login for {}", username);

            let ts = Utc::now();
            let result = client
                .execute(
                    "UPDATE auth SET lastlogin=$1 WHERE username=$2;",
                    &[&ts, &username],
                )
                .await?;

            if result < 1 {
                warn!("failed to update lastlogin for user {}", username);
            }

            Ok(true)
        } else {
            warn!("login for {} failed: wrong password", username);
            Ok(false)
        }
    } else {
        warn!("login for {} failed: account does not exist", username);
        Err(SqlError::UserNotExist)
    }
}

pub async fn user_set_enabled(username: &str, enabled: bool) -> Result<()> {
    let client = POOL.get().await?;

    let result = client
        .execute(
            "UPDATE auth SET enabled=$1 WHERE username=$2;",
            &[&enabled, &username],
        )
        .await?;

    if result < 1 {
        Err(SqlError::UserNotExist)
    } else {
        Ok(())
    }
}

pub async fn user_enabled(username: &str) -> Result<bool> {
    let client = POOL.get().await?;

    let rows = client
        .query(
            "SELECT enabled FROM auth WHERE username=$1 LIMIT 1;",
            &[&username],
        )
        .await?;

    if !rows.is_empty() {
        let row = &rows[0];

        let enabled: bool = row.get("enabled");

        Ok(enabled)
    } else {
        Err(SqlError::UserNotExist)
    }
}

pub async fn user_create(username: &str, password: &str) -> Result<()> {
    let client = POOL.get().await?;

    let sqlpasswd = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;

    let result = client
        .execute(
            "INSERT INTO auth (username, passwd, enabled)
        VALUES($1, $2, $3);",
            &[&username, &sqlpasswd, &true],
        )
        .await?;

    if result < 1 {
        return Err(SqlError::UserCreateFailed);
    }

    Ok(())
}

pub async fn has_users() -> Result<bool> {
    let client = POOL.get().await?;

    let rows = client
        .query("SELECT username FROM auth LIMIT 1;", &[])
        .await?;

    Ok(rows.len() > 0)
}
