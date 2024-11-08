use crate::models::{self, ClientSearchResult, Search, SearchResult, SearchType};
use crate::{conf, constants};
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use std::time::Duration;
use tokio_postgres::NoTls;
pub type Result<T> = std::result::Result<T, SqlError>;

pub mod client;
pub mod user;
pub mod webhooks;

#[derive(Debug, Error)]
pub enum SqlError {
    #[error("TokioPostgres({0})")]
    TokioPostgres(#[from] tokio_postgres::Error),

    #[error("DeadPoolPostgres({0})")]
    DeadPoolPostgres(#[from] deadpool::managed::PoolError<tokio_postgres::Error>),

    #[error("Bcrypt({0})")]
    Bcrypt(#[from] bcrypt::BcryptError),

    #[error("SqlError(user does not exist)")]
    UserNotExist,

    #[error("SqlError(user is disabled)")]
    UserDisabled,

    #[error("SqlError(user creation failed)")]
    UserCreateFailed,

    #[error("SqlError(client name {0} already exists)")]
    ClientNameExists(String),

    #[error("SqlError(Client does not exist id: {0})")]
    ClientNotExist(String),

    #[error("SqlError(No schedule for search {0}")]
    NoSuchSchedule(i32),
}

impl actix_web::ResponseError for SqlError {}

lazy_static! {
    // Postres Pool all functions get their client from
    static ref POOL: Pool = create_pool().unwrap();
}

fn create_pool() -> Result<deadpool_postgres::Pool> {
    let pg_params = conf::get_pg_params().expect("failed to get pg_params");

    let config: tokio_postgres::Config = pg_params.parse::<tokio_postgres::Config>()?;

    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };

    let mgr = Manager::from_config(config, NoTls, mgr_config);
    let pool = Pool::builder(mgr).max_size(16).build().unwrap();

    Ok(pool)
}

/**
 * Initialize the database. Will automatically update the database
 */
pub async fn initialize_db() -> Result<()> {
    /*
    As db changes, we will check dbinfo to make sure
    dbver matches current. If not we run the incremental
    upgrade functions to update, i.e. update_v1_to_v2()
    then update_v2_to_v3()
    */
    let dbver = get_db_version().await?;
    warn!("Database version {}", dbver);

    match dbver {
        0 => create_db_tables().await?,
        1 => {
            // current version
        }
        _ => {
            error!("unknown database version: {}", dbver);
            panic!("unknown database version: {}", dbver);
        }
    }

    Ok(())
}

pub async fn get_db_version() -> Result<i32> {
    if !table_exists("dbinfo").await? {
        return Ok(0);
    }
    let client = POOL.get().await?;

    let rows = client
        .query("SELECT dbver FROM dbinfo LIMIT 1;", &[])
        .await?;

    let dbver = if rows.is_empty() {
        0
    } else {
        rows[0].get("dbver")
    };

    Ok(dbver)
}

async fn create_db_tables() -> Result<()> {
    warn!("Creating db tables!");
    let mut client = POOL.get().await?;
    let tran = client.transaction().await?;

    tran.execute(
        "CREATE TABLE dbinfo (
            id SERIAL PRIMARY KEY,
            dbver INT
         );",
        &[],
    )
    .await?;
    tran.execute("INSERT INTO dbinfo (dbver) VALUES(1);", &[])
        .await?;

    tran.execute(
        "CREATE TABLE auth (
            username TEXT PRIMARY KEY,
            passwd TEXT NOT NULL,
            lastlogin TIMESTAMP WITH TIME ZONE,
            enabled BOOL NOT NULL
        );",
        &[],
    )
    .await?;

    // meant for preventing authentication brute forcing
    // not implemented yet
    tran.execute(
        "CREATE TABLE authbrute (
            id TEXT PRIMARY KEY,
            ip TEXT NOT NULL,
            ts TIMESTAMP WITH TIME ZONE,
            count INT NOT NULL
        );",
        &[],
    )
    .await?;

    tran.execute(
        "CREATE TABLE clients (
            id TEXT PRIMARY KEY,
            token TEXT NOT NULL,
            name TEXT NOT NULL,
            enabled BOOL NOT NULL,
            created TIMESTAMP WITH TIME ZONE,
            lastconnect TIMESTAMP WITH TIME ZONE
        );",
        &[],
    )
    .await?;

    tran.execute(
        "CREATE TABLE client_schedule (
            id TEXT PRIMARY KEY,
            lastrun TIMESTAMP WITH TIME ZONE,
            manualrun BOOL
        );",
        &[],
    )
    .await?;

    tran.execute(
        "CREATE TABLE scan_schedule (
            searchid INT NOT NULL,
            schedule INT NOT NULL,
            manual BOOL NOT NULL
        );",
        &[],
    )
    .await?;
    // default scan every 30 minutes
    tran.execute(
        "INSERT INTO scan_schedule (searchid, schedule, manual) VALUES(0, 30, 'f');",
        &[],
    )
    .await?;

    /*
     * 'searches' table is for storing log searches.
     */
    tran.execute(
        "CREATE TABLE searches (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            type INT NOT NULL,
            search TEXT NOT NULL,
            locations TEXT [],
            enabled BOOL NOT NULL
        );",
        &[],
    )
    .await?;

    tran.execute(
        "CREATE TABLE search_results (
            id SERIAL PRIMARY KEY,
            client TEXT NOT NULL,
            search INT NOT NULL,
            location TEXT,
            found TEXT [] NOT NULL,
            started TIMESTAMP WITH TIME ZONE NOT NULL
        );",
        &[],
    )
    .await?;

    tran.execute(
        "CREATE TABLE webhooks (
            name TEXT PRIMARY KEY,
            url TEXT NOT NULL,
            username TEXT NOT NULL
        );",
        &[],
    )
    .await?;

    tran.commit().await?;

    Ok(())
}

async fn table_exists(table_name: &str) -> Result<bool> {
    let client = POOL.get().await?;

    let row = client
        .query_one(
            "SELECT EXISTS (SELECT FROM pg_tables WHERE schemaname=$1 AND tablename=$2);",
            &[&"public", &table_name],
        )
        .await?;

    Ok(row.get(0))
}

fn random_string(len: usize) -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    let randstr: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect();

    randstr
}

pub async fn get_search(id: i32) -> Result<Option<models::Search>> {
    let client = POOL.get().await?;

    let rows = client
        .query("SELECT * FROM searches WHERE id=$1 LIMIT 1;", &[&id])
        .await?;

    if !rows.is_empty() {
        let row = &rows[0];
        let locations: Vec<String> = row.get("locations");

        if let Some(stype) = models::SearchType::from_sql_code(row.get("type")) {
            if let Some(search) = row.get("search") {
                return Ok(Some(models::Search {
                    id,
                    name: row.get("name"),
                    stype,
                    search,
                    locations,
                }));
            }
        }
    }

    Ok(None)
}

pub async fn get_searches() -> Result<Vec<models::Search>> {
    let client = POOL.get().await?;

    let rows = client
        .query("SELECT * FROM searches WHERE enabled='t';", &[])
        .await?;

    let mut searches: Vec<models::Search> = Vec::new();
    for row in rows {
        let id: i32 = row.get("id");
        let name: String = row.get("name");
        let stype = models::SearchType::from_sql_code(row.get::<&str, i32>("type")).unwrap();
        let search_str: String = row.get("search");
        let locations: Vec<String> = row.get("locations");

        let search = models::Search::new(id, name, stype, search_str, locations);
        searches.push(search);
    }

    Ok(searches)
}

pub async fn delete_search(id: i32) -> Result<()> {
    let client = POOL.get().await?;

    let _result = client
        .execute("DELETE FROM searches WHERE id=$1;", &[&id])
        .await?;

    Ok(())
}

/**
 * Insert the search object into the database.
 * Returns the new id associated with the search.
 */
pub async fn insert_search(
    name: &str,
    stype: &SearchType,
    search: &str,
    locations: &[String],
) -> Result<i32> {
    let client = POOL.get().await?;

    let rows = client
        .query(
            "INSERT INTO searches
        (name, type, search, locations, enabled)
        VALUES($1, $2, $3, $4, $5)
        RETURNING id;",
            &[&name, &stype.sql_code(), &search, &locations, &true],
        )
        .await?;
    let id: i32 = rows[0].get("id");

    Ok(id)
}

pub async fn insert_client_search_result(
    clientid: &str,
    result: &ClientSearchResult,
) -> Result<()> {
    let client = POOL.get().await?;

    let _result = client
        .execute(
            "INSERT INTO search_results
        (client, search, location, found, started)
        VALUES($1, $2, $3, $4, $5);",
            &[
                &clientid,
                &result.search_id,
                &result.location,
                &result.found,
                &result.started,
            ],
        )
        .await?;

    Ok(())
}

use chrono::{DateTime, Utc};
pub async fn get_search_results(
    clientid: Option<String>,
    before: Option<DateTime<Utc>>,
    after: Option<DateTime<Utc>>,
) -> Result<Vec<SearchResult>> {
    let results = get_all_search_results().await?;
    let mut new_results: Vec<SearchResult> = Vec::new();

    for result in results {
        let mut include = true;
        if let Some(clientid) = &clientid {
            if &result.client_id != clientid {
                include = false;
            }
        }

        if let Some(before) = &before {
            if &result.started <= before {
                include = false;
            }
        }
        if let Some(after) = &after {
            if &result.started >= after {
                include = false;
            }
        }
        if include {
            new_results.push(result);
        }
    }

    Ok(new_results)
}

pub async fn get_all_search_results() -> Result<Vec<SearchResult>> {
    let client = POOL.get().await?;

    let rows = client.query("SELECT * FROM search_results;", &[]).await?;

    let mut results: Vec<SearchResult> = Vec::new();

    for row in rows {
        let search_id: i32 = row.get("search");
        let search_name: String = {
            let rows = client
                .query(
                    "SELECT name FROM searches WHERE id=$1 LIMIT 1;",
                    &[&search_id],
                )
                .await?;

            rows[0].get("name")
        };
        let client_id: String = row.get("client");
        let client_name: String = {
            let rows = client
                .query(
                    "SELECT name FROM clients WHERE id=$1 LIMIT 1;",
                    &[&client_id],
                )
                .await?;

            rows[0].get("name")
        };

        let result = SearchResult {
            client_id,
            client_name,
            search_id,
            search_name,
            location: row.get("location"),
            found: row.get("found"),
            started: row.get("started"),
        };
        results.push(result);
    }

    Ok(results)
}

#[derive(Debug)]
pub struct ScanSchedule {
    pub dur: Duration,
    pub manual: bool,
}
impl ScanSchedule {
    pub fn get_interval(&self) -> Duration {
        self.dur
    }
    pub fn is_manual(&self) -> bool {
        self.manual
    }
}
pub async fn get_scan_schedule() -> Result<ScanSchedule> {
    let client = POOL.get().await?;

    let rows = client
        .query(
            "SELECT * FROM scan_schedule WHERE searchid=$1 LIMIT 1;",
            &[&0i32],
        )
        .await?;

    if !rows.is_empty() {
        let row = &rows[0];
        let schedule: i32 = row.get("schedule");
        let duration = Duration::from_secs((schedule as u64) * 60);

        Ok(ScanSchedule {
            dur: duration,
            manual: row.get("manual"),
        })
    } else {
        Err(SqlError::NoSuchSchedule(0i32))
    }
}

pub async fn set_search_schedule_minutes(schedule: i32, manual: bool) -> Result<()> {
    let client = POOL.get().await?;

    let _result = client
        .execute(
            "UPDATE scan_schedule SET schedule=$1, manual=$3 WHERE searchid=$2;",
            &[&schedule, &0i32, &manual],
        )
        .await?;

    Ok(())
}
