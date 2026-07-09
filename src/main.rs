use std::{
    env,
    process::{self, Command},
};

use fr24_detector::{
    CHECK_TIME_FREQUENCY, CliCommand, VERSION, check_config_file, config_general_webhook_remove,
    config_general_webhook_set, parse_cli_command, sync_problem_states_from_output,
};

/// Program entrypoint
fn main() {
    // Run this program only on Linux
    if cfg!(not(target_os = "linux")) {
        println!("This program runs only on Linux!");
        return;
    }

    // Send a message if running in debug mode
    if cfg!(debug_assertions) {
        println!("WARNING: Running in debug mode");
    }

    let args: Vec<String> = env::args().skip(1).collect();

    match parse_cli_command(&args) {
        Ok(CliCommand::Version) => {
            println!("{}", VERSION);
            return;
        }
        Ok(CliCommand::ConfigWebhookSet(url)) => {
            if let Err(err) = check_config_file() {
                eprintln!("Error: {}", err);
                process::exit(1);
            }
            if let Err(err) = config_general_webhook_set(&url) {
                eprintln!("Error: {}", err);
                process::exit(1);
            }
            println!("Webhook URL updated.");
            return;
        }
        Ok(CliCommand::ConfigWebhookDelete) => {
            if let Err(err) = check_config_file() {
                eprintln!("Error: {}", err);
                process::exit(1);
            }
            if let Err(err) = config_general_webhook_remove() {
                eprintln!("Error: {}", err);
                process::exit(1);
            }
            println!("Webhook URL removed.");
            return;
        }
        Ok(CliCommand::Help) => {
            println!(
                "Usage: fr24d version | fr24d config webhook set <url> | fr24d config webhook delete"
            );
            return;
        }
        Ok(CliCommand::Daemon) => {}
        Err(err) => {
            eprintln!("{}", err);
            process::exit(2);
        }
    }

    // Check system uptime. If it's less than a minute, wait a minute before continuing
    match fr24_detector::get_system_uptime() {
        Ok(uptime) => {
            if uptime < 60 {
                println!("System uptime is less than a minute. Waiting for a minute...");
                std::thread::sleep(std::time::Duration::from_secs(60));
            }
        }
        Err(e) => {
            println!("Error checking system uptime: {}", e);
            return;
        }
    }

    // Make it so that it runs the check every minute
    loop {
        if let Err(err) = check_failed_checks() {
            eprintln!("Error checking status: {}", err);
        }

        std::thread::sleep(std::time::Duration::from_secs(CHECK_TIME_FREQUENCY));
    }
}

/// Checks for any failed parts in the fr24feed-status command
fn check_failed_checks() -> Result<(), String> {
    let output = Command::new("fr24feed-status")
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "fr24feed-status exited with error: {}",
            stderr.trim()
        ));
    }

    let content = output.stdout;
    let status_output = String::from_utf8_lossy(&content);
    let (failed_checks, notification) = sync_problem_states_from_output(&status_output)?;

    if failed_checks.is_empty() {
        println!("All checks are passing.");
    } else {
        println!("Failed checks:");
        for check in failed_checks {
            println!("- {}", check);
        }
    }

    if let Some(message) = notification {
        println!("Webhook notification sent: {}", message);
    }

    Ok(())
}
