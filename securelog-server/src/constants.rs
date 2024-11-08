pub const CONFIG_SERVER_CERT: &str = "cert";
pub const CONFIG_SERVER_KEY: &str = "key";
pub const CONFIG_SERVER_LISTEN: &str = "listen";
pub const CONFIG_PG_PARAMS: &str = "pg_params";
pub const CONFIG_LOG_DIR: &str = "log_dir";
pub const CONFIG_LOG_LEVEL: &str = "log_level";
pub const CONFIG_LOG_STDOUT: &str = "log_stdout";

// only available with 'debug' feature enabled
pub const CONFIG_SERVER_HTTPS: &str = "https";

pub const SEARCH_REGEX: i32 = 1;
pub const SEARCH_CONTAINS: i32 = 2;
pub const SEARCH_WILDCARD: i32 = 3;
