#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate thiserror;

mod conf;
mod constants;
mod models;
mod sql;
mod web;
mod webhooks;

#[actix_web::main]
async fn main() {
    use clap::Arg;
    use clap::Command;
    let matches = command!()
        .subcommand(Command::new("show-config"))
        .subcommand(
            Command::new("create-user").arg(
                Arg::new("username")
                    .long("username")
                    .short('u')
                    .takes_value(true)
                    .required(false),
            ),
        )
        .subcommand(Command::new("initialize-db").about("Initialize database or update database"))
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .required(false)
                .takes_value(true),
        )
        .get_matches();

    if let Some(loc) = matches.value_of("config") {
        std::env::set_var("CONFIG_LOCATION", loc);
    }

    conf::initialize_config();

    if matches.subcommand_matches("show-config").is_some() {
        use std::collections::HashMap;

        let config = conf::read_config().unwrap();
        println!(
            "{:?}",
            config.try_deserialize::<HashMap<String, String>>().unwrap()
        );

        std::process::exit(0);
    }

    setup_log().unwrap();

    // will check database, initialize/update if needed
    sql::initialize_db().await.unwrap();

    if let Some(_matches) = matches.subcommand_matches("initialize-db") {
        info!("Database initialized, exiting!");
        std::process::exit(0);
    }

    if let Some(smatches) = matches.subcommand_matches("create-user") {
        let username = match smatches.value_of("username") {
            Some(username) => username.to_string(),
            None => prompt_user_input("Username: ").unwrap(),
        };
        let password = rpassword::prompt_password("Password: ").unwrap();
        let password2 = rpassword::prompt_password("Password again: ").unwrap();

        if password != password2 {
            panic!("Passwords do not match!");
        }

        sql::user::user_create(&username, &password).await.unwrap();

        info!("created user {}", username);
        std::process::exit(0);
    }

    if !sql::user::has_users().await.unwrap() {
        create_first_user().await.unwrap();
    }

    webhooks::send_message("starting up!").await.unwrap();
    web::start().await.unwrap();
}

fn prompt_user_input(prompt: &str) -> std::io::Result<String> {
    use std::io;
    use std::io::Write;
    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input = String::new();

    io::stdin().read_line(&mut input)?;

    if input.ends_with('\r') {
        input.pop();
    }
    if input.ends_with('\n') {
        input.pop();
    }

    Ok(input)
}

fn setup_log() -> anyhow::Result<()> {
    use flexi_logger::{opt_format, Duplicate, FileSpec, Logger};

    let log_level = conf::get_log_level().unwrap_or_else(|_| String::from("info"));

    let log_dir = conf::get_log_dir().unwrap_or_else(|_| String::from("logs/"));

    let mut logger = Logger::try_with_str(log_level)?.format(opt_format);
    if let Ok(true) = conf::get_log_stdout() {
        logger = logger.duplicate_to_stderr(Duplicate::All);
    }

    // first check permissions
    use std::fs;

    if let Ok(dir) = fs::metadata(&log_dir) {
        let perm = dir.permissions();
        if perm.readonly() {
            error!("cannot write logs to {}", log_dir);
            panic!("cannot write logs to {}", log_dir);
        }
    } else {
        fs::create_dir_all(&log_dir)?;
    }

    logger = logger.log_to_file(FileSpec::default().directory(&log_dir));

    if cfg!(windows) {
        logger = logger.use_windows_line_ending();
    }

    logger.start()?;

    Ok(())
}

async fn create_first_user() -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        if std::os::unix::process::parent_id() == 1 {
            warn!("running under systemd, not asking to create user..");
            return Ok(());
        }
    }
    use std::io::Write;

    std::io::stdout().flush()?;
    println!("No users in database, creating first user");

    let username = prompt_user_input("Username: ")?;
    let password1 = rpassword::prompt_password("Password: ")?;
    let password2 = rpassword::prompt_password("Password again: ")?;

    if password1 == password2 {
        sql::user::user_create(&username, &password1).await?;
        println!("User created!");
    } else {
        panic!("Passwords do not match! Failed to create first user!");
    }

    Ok(())
}
