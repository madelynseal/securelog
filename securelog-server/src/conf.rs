use crate::constants;
use config::{Config, ConfigError};
use std::sync::RwLock;

// see contants.rs for config option names
lazy_static! {
    static ref CONFIG: RwLock<Config> = RwLock::new(Config::default());
}
pub fn initialize_config() {
    let config = read_config().unwrap();

    let mut val = CONFIG.write().unwrap();

    *val = config;
}

pub fn read_config() -> Result<Config, config::ConfigError> {
    let mut builder = Config::builder().add_source(config::Environment::with_prefix("SERVER"));

    if let Ok(loc) = std::env::var("CONFIG_LOCATION") {
        builder = builder.add_source(config::File::with_name(&loc));
    }

    let settings = builder.build()?;

    check_config(&settings);

    Ok(settings)
}

fn check_config(config: &Config) {
    check_config_exists(config, constants::CONFIG_SERVER_CERT);
    check_config_exists(config, constants::CONFIG_SERVER_KEY);
    check_config_exists(config, constants::CONFIG_SERVER_LISTEN);

    // database config options
    check_config_exists(config, constants::CONFIG_PG_PARAMS);
}

fn check_config_exists(config: &Config, key: &str) {
    match config.get_string(key) {
        Ok(_) => (),
        Err(e) => {
            warn!("error getting config option {}: {}", key, e);
        }
    }
}

pub fn get_server_cert() -> Result<String, ConfigError> {
    let config = CONFIG.read().unwrap();

    config.get_string(constants::CONFIG_SERVER_CERT)
}
pub fn get_server_cert_key() -> Result<String, ConfigError> {
    let config = CONFIG.read().unwrap();

    config.get_string(constants::CONFIG_SERVER_KEY)
}
pub fn get_server_listen() -> Result<String, ConfigError> {
    let config = CONFIG.read().unwrap();

    config.get_string(constants::CONFIG_SERVER_LISTEN)
}
pub fn get_pg_params() -> Result<String, ConfigError> {
    let config = CONFIG.read().unwrap();

    config.get_string(constants::CONFIG_PG_PARAMS)
}
pub fn get_log_dir() -> Result<String, ConfigError> {
    let config = CONFIG.read().unwrap();

    config.get_string(constants::CONFIG_LOG_DIR)
}
pub fn get_log_level() -> Result<String, ConfigError> {
    let config = CONFIG.read().unwrap();

    config.get_string(constants::CONFIG_LOG_LEVEL)
}
pub fn get_log_stdout() -> Result<bool, ConfigError> {
    let config = CONFIG.read().unwrap();

    config.get_bool(constants::CONFIG_LOG_STDOUT)
}
pub fn get_use_https() -> Result<bool, ConfigError> {
    let config = CONFIG.read().unwrap();

    config.get_bool(constants::CONFIG_SERVER_HTTPS)
}
