use fr24_detector::{parse_cli_command, CliCommand};

#[test]
fn parses_version_command() {
    let args = vec!["version".to_string()];

    assert!(matches!(parse_cli_command(&args), Ok(CliCommand::Version)));
}

#[test]
fn parses_webhook_set_command() {
    let args = vec![
        "config".to_string(),
        "webhook".to_string(),
        "set".to_string(),
        "https://example.test".to_string(),
    ];

    assert!(matches!(
        parse_cli_command(&args),
        Ok(CliCommand::ConfigWebhookSet(url)) if url == "https://example.test"
    ));
}
