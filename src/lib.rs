// Import the constants module
mod consts_fr24d;

use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use json_escape::escape_str;
use rusqlite::{Connection, OptionalExtension};

pub const VERSION: &str = consts_fr24d::VERSION;

// -----------------------------------------------------------------------------
// CLI parsing
// -----------------------------------------------------------------------------

pub enum CliCommand {
    Daemon,
    Version,
    Help,
    ConfigWebhookSet(String),
    ConfigWebhookDelete,
}

/// Parse command-line arguments into the supported CLI commands.
pub fn parse_cli_command(args: &[String]) -> Result<CliCommand, String> {
    match args.first().map(String::as_str) {
        None => Ok(CliCommand::Daemon),
        Some("version") => Ok(CliCommand::Version),
        Some("help") => Ok(CliCommand::Help),
        Some("config") => match (args.get(1).map(String::as_str), args.get(2).map(String::as_str), args.get(3).map(String::as_str)) {
            (Some("webhook"), Some("set"), Some(url)) => Ok(CliCommand::ConfigWebhookSet(url.to_string())),
            (Some("webhook"), Some("delete"), None) => Ok(CliCommand::ConfigWebhookDelete),
            _ => Err("Usage: fr24d version | fr24d config webhook set <url> | fr24d config webhook delete".to_string()),
        },
        Some(other) => Err(format!("Unknown command: {}", other)),
    }
}

// -----------------------------------------------------------------------------
// Status parsing
// -----------------------------------------------------------------------------

/// Extract the names of checks that reported as failed from the status output.
pub fn parse_failed_checks(status_output: &str) -> Vec<String> {
    status_output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.contains("failed!") {
                trimmed
                    .split_once(':')
                    .map(|(label, _)| label.trim().to_string())
            } else {
                None
            }
        })
        .collect()
}

/// The current state of the monitored problems.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ProblemStates {
    pub link: bool,
    pub receiver: bool,
    pub started: bool,
}

/// Convert the failed-check names into a set of booleans describing which problems are active.
pub fn derive_problem_states(failed_checks: &[String]) -> ProblemStates {
    let mut states = ProblemStates::default();

    for check in failed_checks {
        match check.to_lowercase().as_str() {
            "fr24 link" | "link" => states.link = true,
            "receiver" => states.receiver = true,
            _ => {}
        }
    }

    states.started = !failed_checks.is_empty();
    states
}

// -----------------------------------------------------------------------------
// Configuration and persistence
// -----------------------------------------------------------------------------

/// Return the path to the SQLite database used for persistence.
pub fn get_config_path() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(consts_fr24d::CONFIG_FILENAME))
        .unwrap_or_else(|| PathBuf::from(consts_fr24d::CONFIG_FILENAME))
}

/// Open the SQLite database and ensure the settings table exists.
fn open_database(path: &Path) -> Result<Connection, String> {
    let conn = Connection::open(path).map_err(|e| format!("Failed to open database: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        [],
    )
    .map_err(|e| format!("Failed to initialize settings table: {}", e))?;

    Ok(conn)
}

/// Read a single stored setting from the database.
fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
        row.get::<_, String>(0)
    })
    .optional()
    .map_err(|e| format!("Failed to read setting {}: {}", key, e))
}

/// Write or update a single stored setting in the database.
fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [key, value],
    )
    .map_err(|e| format!("Failed to write setting {}: {}", key, e))?;

    Ok(())
}

// -----------------------------------------------------------------------------
// State syncing and webhook handling
// -----------------------------------------------------------------------------

/// Sync the latest problem state to the database and decide whether to send a webhook.
pub fn sync_problem_states_from_output(
    status_output: &str,
) -> Result<(Vec<String>, Option<String>), String> {
    sync_problem_states_from_output_with_path(get_config_path(), status_output)
}

/// Same as sync_problem_states_from_output but uses an explicit database path.
pub fn sync_problem_states_from_output_with_path(
    config_path: impl AsRef<Path>,
    status_output: &str,
) -> Result<(Vec<String>, Option<String>), String> {
    let config_path = config_path.as_ref();
    let failed_checks = parse_failed_checks(status_output);
    let states = derive_problem_states(&failed_checks);

    ensure_config_file_exists(config_path)?;

    let conn = open_database(config_path)?;
    let previous_states = ProblemStates {
        link: parse_bool_setting(&conn, "problems_link")?,
        receiver: parse_bool_setting(&conn, "problems_receiver")?,
        started: parse_bool_setting(&conn, "problems_started")?,
    };
    let startup_notified = parse_bool_setting(&conn, "startup_notified")?;

    write_problem_state(&conn, "problems_link", states.link)?;
    write_problem_state(&conn, "problems_receiver", states.receiver)?;
    write_problem_state(&conn, "problems_started", states.started)?;
    set_setting(&conn, "startup_notified", "true")?;

    let webhook_url = read_config_value(&conn, "webhook_url")?;
    let should_notify = !startup_notified || previous_states != states;

    let notification_message = if !startup_notified {
        "FR24 detector is online.".to_string()
    } else if states.started {
        format!(
            "FR24 detector detected problems: {}",
            failed_checks.join(", ")
        )
    } else {
        "FR24 detector recovered: all checks are passing.".to_string()
    };

    if webhook_url.trim().is_empty() || !should_notify {
        return Ok((failed_checks, None));
    }

    match send_webhook_message(&webhook_url, &notification_message) {
        Ok(()) => Ok((failed_checks, Some(notification_message))),
        Err(err) => Err(err),
    }
}

/// Ensure the database exists, creating it and migrating any legacy data if needed.
fn ensure_config_file_exists(config_path: &Path) -> Result<(), String> {
    if config_path.exists() {
        return Ok(());
    }

    create_config_file_at_path(config_path)
}

/// Create a fresh settings database with default values.
fn create_config_file() -> Result<(), String> {
    create_config_file_at_path(&get_config_path())
}

/// Create the database at the given path with default values.
fn create_config_file_at_path(config_path: &Path) -> Result<(), String> {
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create parent directory for configuration file: {}",
                e
            )
        })?;
    }

    let conn = open_database(config_path)?;
    set_setting(&conn, "webhook_url", "")?;
    set_setting(&conn, "problems_link", "false")?;
    set_setting(&conn, "problems_receiver", "false")?;
    set_setting(&conn, "problems_started", "false")?;
    set_setting(&conn, "startup_notified", "false")?;

    Ok(())
}

/// Persist a single problem-state boolean to the database.
fn write_problem_state(conn: &Connection, key: &str, value: bool) -> Result<(), String> {
    set_setting(conn, key, if value { "true" } else { "false" })
}

/// Read a boolean-valued setting from the database.
fn parse_bool_setting(conn: &Connection, key: &str) -> Result<bool, String> {
    match get_setting(conn, key)? {
        Some(value) => match value.trim().to_lowercase().as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Ok(false),
        },
        None => Ok(false),
    }
}

/// Read a text-valued setting from the database.
fn read_config_value(conn: &Connection, key: &str) -> Result<String, String> {
    get_setting(conn, key).map(|value| value.unwrap_or_default())
}

/// Send a JSON payload to the configured webhook URL.
fn send_webhook_message(webhook_url: &str, message: &str) -> Result<(), String> {
    let escaped_message = escape_str(message);
    let payload = format!("{{\"content\":\"{}\"}}", escaped_message);

    let output = Command::new("curl")
        .args([
            "-sS",
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "--data",
            &payload,
            webhook_url,
        ])
        .output()
        .map_err(|e| format!("Failed to execute curl for webhook: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Webhook request failed: {}", stderr.trim()))
    }
}

// -----------------------------------------------------------------------------
// Config helpers
// -----------------------------------------------------------------------------

/// Ensure the settings database exists before reading or updating it.
pub fn check_config_file() -> Result<(), String> {
    if get_config_path().exists() {
        Ok(())
    } else {
        create_config_file()
    }
}

/// Return the stored webhook URL from the database.
pub fn config_general_webhook_get() -> Result<String, String> {
    check_config_file()?;

    let conn = open_database(&get_config_path())?;
    get_setting(&conn, "webhook_url")?
        .map(|value| value)
        .ok_or_else(|| "webhook_url not found in configuration database".to_string())
}

/// Store a new webhook URL in the database.
pub fn config_general_webhook_set(new_url: &str) -> Result<(), String> {
    check_config_file()?;

    let conn = open_database(&get_config_path())?;
    set_setting(&conn, "webhook_url", new_url)
}

/// Clear the stored webhook URL from the database.
pub fn config_general_webhook_remove() -> Result<(), String> {
    check_config_file()?;

    let conn = open_database(&get_config_path())?;
    set_setting(&conn, "webhook_url", "")
}

// -----------------------------------------------------------------------------
// System helpers
// -----------------------------------------------------------------------------

/// Read the current system uptime in seconds.
/// This is used to avoid sending an initial down-state alert right after boot.
pub fn get_system_uptime() -> Result<u64, String> {
    let uptime_path = "/proc/uptime";
    let uptime_content = std::fs::read_to_string(uptime_path)
        .map_err(|e| format!("Failed to read {}: {}", uptime_path, e))?;

    let uptime_seconds = uptime_content
        .split_whitespace()
        .next()
        .ok_or_else(|| "Failed to parse uptime".to_string())?
        .parse::<f64>()
        .map_err(|e| format!("Failed to parse uptime as float: {}", e))?;

    Ok(uptime_seconds as u64)
}
