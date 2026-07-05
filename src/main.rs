use std::{
    env,
    ops::Not,
    process::{self, Command},
};

use fr24_detector::{
    CliCommand, VERSION, check_config_file, config_general_webhook_remove,
    config_general_webhook_set, parse_cli_command, parse_failed_checks,
};

fn main() {
    // Run this program only on Linux
    if cfg!(not(target_os = "linux")) {
        println!("This program runs only on Linux!");
        return;
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
        let failed_checks = check_failed_checks();
        if failed_checks.is_empty().not() {
            println!("Failed checks:");
            for check in failed_checks {
                println!("- {}", check);
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}

fn check_failed_checks() -> Vec<String> {
    let output = Command::new("fr24feed-status")
        .output()
        .expect("Failed to execute command");
    let content = output.stdout;
    let status_output = String::from_utf8_lossy(&content);

    parse_failed_checks(&status_output)
}
