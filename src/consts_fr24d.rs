pub const CONFIG_FILENAME: &str = ".fr24detector.sqlite3";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CHECK_TIME_FREQUENCY: u64 = if cfg!(debug_assertions) { 5 } else { 60 };