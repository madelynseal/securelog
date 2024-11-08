use crate::constants;
use config::{Config, ConfigError};
use std::sync::RwLock;

// see contants.rs for config option names
lazy_static! {
    pub static ref CONFIG: RwLock<Config> = RwLock::new(Config::default());
}

pub fn read_config() -> Result<Config, config::ConfigError> {
    let mut builder = Config::builder().add_source(config::Environment::with_prefix("CLIENT"));

    if let Ok(loc) = std::env::var("CONFIG_LOCATION") {
        builder = builder.add_source(config::File::with_name(&loc));
    }

    let settings = builder.build()?;

    Ok(settings)
}
pub fn initialize_config() -> Result<(), config::ConfigError> {
    let config = read_config()?;

    let mut lock = CONFIG.write().unwrap();

    *lock = config;

    Ok(())
}

pub fn get_id() -> Result<String, ConfigError> {
    let config = CONFIG.read().unwrap();

    config.get_string(constants::CONFIG_ID)
}

pub fn get_token() -> Result<String, ConfigError> {
    let config = CONFIG.read().unwrap();

    config.get_string(constants::CONFIG_TOKEN)
}

pub fn get_server() -> Result<String, ConfigError> {
    let config = CONFIG.read().unwrap();

    config.get_string(constants::CONFIG_SERVER)
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
