extern crate config;
extern crate reqwest;
#[macro_use]
extern crate lazy_static;
extern crate anyhow;
#[macro_use]
extern crate thiserror;
#[macro_use]
extern crate log;
extern crate flexi_logger;

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate clap;

mod conf;
mod constants;
mod models;
mod searchrunner;
mod webclient;

use std::thread::sleep;
use std::time::Duration;

fn main() {
    use clap::Arg;

    let matches = command!()
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
    conf::initialize_config().unwrap();
    check_initialized();

    setup_log().unwrap();

    use std::env;
    info!("git commit: {:?}", env::var("VERGEN_GIT_SHA"));
    info!("build date: {:?}", env::var("VERGEN_BUILD_TIMESTAMP"));
    info!(
        "build host triple: {:?}",
        env::var("VERGEN_RUSTC_HOST_TRIPLE")
    );
    info!("build rust version: {:?}", env::var("VERGEN_RUSTC_SEMVER"));

    if !webclient::login().unwrap() {
        panic!("failed to login!");
    }

    loop {
        if webclient::get_should_run().unwrap_or(false) {
            webclient::notify_running().unwrap();
            match searchrunner::run_once() {
                Ok(_) => (),
                Err(e) => {
                    warn!("error running search: {}", e);
                }
            }
        }
        sleep(Duration::from_secs(60));
    }

    //webclient::logout().unwrap();
}

fn setup_log() -> anyhow::Result<()> {
    use flexi_logger::{opt_format, Duplicate, FileSpec, Logger};

    let logdir = if let Ok(dir) = conf::get_log_dir() {
        dir
    } else {
        String::from("logs/")
    };

    let log_level = if let Ok(level) = conf::get_log_level() {
        level
    } else {
        String::from("info")
    };

    let mut logger = Logger::try_with_str(log_level)?.format(opt_format);

    if let Ok(true) = conf::get_log_stdout() {
        logger = logger.duplicate_to_stderr(Duplicate::All);
    }

    // first check permissions
    use std::fs;

    if let Ok(dir) = fs::metadata(&logdir) {
        let perm = dir.permissions();
        if perm.readonly() {
            error!("cannot write logs to {}", &logdir);
            panic!("cannot write logs to {}", &logdir);
        }
    } else {
        fs::create_dir_all(&logdir)?;
    }

    logger = logger.log_to_file(FileSpec::default().directory(&logdir));

    if cfg!(windows) {
        logger = logger.use_windows_line_ending();
    }

    logger.start()?;

    Ok(())
}

/**
 * Checks if any important config options are missing.
 * Will print missing config options to console
 */
fn config_missing() -> bool {
    let config = conf::CONFIG.read().unwrap();

    if config.get_string(constants::CONFIG_SERVER).is_err() {
        println!("config missing!");
        true
    } else if config.get_string(constants::CONFIG_NAME).is_err() {
        println!("config missing!");
        true
    } else if config.get_string(constants::CONFIG_ID).is_err() {
        println!("config missing!");
        true
    } else if config.get_string(constants::CONFIG_TOKEN).is_err() {
        println!("config missing!");
        true
    } else {
        false
    }
}

fn check_initialized() {
    if config_missing() {
        println!("Client uninitialized, will run init_script!");
        init_script().unwrap();
        std::process::exit(0);
    }
}

#[derive(Debug, Serialize)]
struct TomlConfig {
    pub server: String,
    pub name: String,
    pub id: String,
    pub token: String,
    pub log_dir: String,
    pub log_level: String,
    pub log_stdout: bool,
}
fn init_script() -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        let parent_pid = std::os::unix::process::parent_id();

        if parent_pid == 1 {
            // ran by init script
            println!("Need to create config before running as service!");
            std::process::exit(1);
        }
    }

    let server = prompt_user_input("Server base url: ")?;
    let name = prompt_user_input("Client name: ")?;

    println!("Next up is credentials to login to the server to create an id/token for the client");
    let username = prompt_user_input("Username: ")?;
    let password = rpassword::prompt_password("Password: ")?;

    let client_auth = webclient::create_client_token(&server, &name, &username, &password)?;

    let config = TomlConfig {
        server,
        name,
        id: client_auth.id,
        token: client_auth.token,
        log_dir: String::from("logs"),
        log_level: String::from("info"),
        log_stdout: true,
    };

    let outfile = prompt_user_input("File to save to: ")?;

    let toml = toml::to_string_pretty(&config)?;

    match std::fs::write(&outfile, &toml) {
        Ok(_) => {
            println!("file saved successfully!");
        }
        Err(e) => {
            eprintln!("failed to write to file {}: {}", outfile, e);
            println!("toml config:\n{}", toml);
        }
    }

    Ok(())
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
