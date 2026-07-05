// Import the constants module
mod consts_fr24d;

pub const VERSION: &str = consts_fr24d::VERSION;

pub enum CliCommand {
    Daemon,
    Version,
    ConfigWebhookSet(String),
    ConfigWebhookDelete,
}

pub fn parse_cli_command(args: &[String]) -> Result<CliCommand, String> {
    match args.first().map(String::as_str) {
        None => Ok(CliCommand::Daemon),
        Some("version") => Ok(CliCommand::Version),
        Some("config") => match (args.get(1).map(String::as_str), args.get(2).map(String::as_str), args.get(3).map(String::as_str)) {
            (Some("webhook"), Some("set"), Some(url)) => Ok(CliCommand::ConfigWebhookSet(url.to_string())),
            (Some("webhook"), Some("delete"), None) => Ok(CliCommand::ConfigWebhookDelete),
            _ => Err("Usage: fr24d version | fr24d config webhook set <url> | fr24d config webhook delete".to_string()),
        },
        Some(other) => Err(format!("Unknown command: {}", other)),
    }
}

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

// Configuration file time
pub fn check_config_file() -> Result<(), String> {
    if std::path::Path::new(consts_fr24d::CONFIG_PATH).exists() {
        Ok(())
    } else {
        create_config_file()
    }
}

fn create_config_file() -> Result<(), String> {
    let config_path = consts_fr24d::CONFIG_PATH;

    std::fs::write(config_path, consts_fr24d::DEFAULT_CONFIG_CONTENT)
        .map_err(|e| format!("Failed to create configuration file: {}", e))?;

    Ok(())
}

// Helper functions: webhook url
pub fn config_general_webhook_get() -> Result<String, String> {
    check_config_file()?;

    let config_path = consts_fr24d::CONFIG_PATH;
    let config_content = std::fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read configuration file: {}", e))?;

    for line in config_content.lines() {
        if line.trim_start().starts_with("webhook_url=") {
            return Ok(line.trim_start()[12..].trim().to_string());
        }
    }

    Err("webhook_url not found in configuration file".to_string())
}

fn update_config_line(
    config_path: &str,
    predicate: impl Fn(&str) -> bool,
    replacement: &str,
) -> Result<(), String> {
    let config_content = std::fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read configuration file: {}", e))?;

    let mut found = false;
    let mut lines: Vec<String> = config_content.lines().map(str::to_string).collect();
    for line in &mut lines {
        if predicate(line) {
            *line = replacement.to_string();
            found = true;
            break;
        }
    }

    if !found {
        return Err("webhook_url not found in configuration file".to_string());
    }

    let mut updated_content = lines.join("\n");
    if config_content.ends_with('\n') {
        updated_content.push('\n');
    }

    std::fs::write(config_path, updated_content)
        .map_err(|e| format!("Failed to write configuration file: {}", e))?;

    Ok(())
}

pub fn config_general_webhook_set(new_url: &str) -> Result<(), String> {
    check_config_file()?;

    update_config_line(
        consts_fr24d::CONFIG_PATH,
        |line| line.trim_start().starts_with("webhook_url="),
        &format!("webhook_url={}", new_url),
    )
}

pub fn config_general_webhook_remove() -> Result<(), String> {
    check_config_file()?;

    update_config_line(
        consts_fr24d::CONFIG_PATH,
        |line| line.trim_start().starts_with("webhook_url="),
        "webhook_url=",
    )
}

// Get the system uptime
// This is so that we do not send a message for the link being down if the system has just started
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
