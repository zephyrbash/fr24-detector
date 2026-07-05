use std::{
    env, fs,
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    sync::{Mutex, OnceLock},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use fr24_detector::{
    CliCommand, derive_problem_states, get_config_path, parse_cli_command,
    sync_problem_states_from_output_with_path,
};

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn env_lock() -> &'static Mutex<()> {
    ENV_LOCK.get_or_init(|| Mutex::new(()))
}

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

#[test]
fn derives_problem_states_from_failed_checks() {
    let failed_checks = vec!["FR24 Link".to_string(), "Receiver".to_string()];

    let states = derive_problem_states(&failed_checks);

    assert!(states.link);
    assert!(states.receiver);
    assert!(states.started);
}

#[test]
fn uses_home_directory_for_default_config_path() {
    let _guard = env_lock().lock().unwrap();
    let unique_dir = std::env::temp_dir().join(format!(
        "fr24-detector-home-test-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&unique_dir).unwrap();

    let previous_home = env::var_os("HOME");
    unsafe {
        env::set_var("HOME", &unique_dir);
    }

    let config_path = get_config_path();

    unsafe {
        if let Some(home) = previous_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
    }

    assert_eq!(config_path, unique_dir.join(".fr24detector.sqlite3"));
}

#[test]
fn writes_problem_states_to_config_file() {
    let unique_dir = std::env::temp_dir().join(format!(
        "fr24-detector-test-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&unique_dir).unwrap();

    let config_path = unique_dir.join("fr24detector.sqlite3");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let webhook_url = format!("http://{}", listener.local_addr().unwrap());

    let server_thread = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buffer = [0; 1024];
        let _ = stream.read(&mut buffer).unwrap();
        let response = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
        stream.write_all(response).unwrap();
    });

    let path = PathBuf::from(&config_path);
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)",
        ["webhook_url", &webhook_url],
    )
    .unwrap();

    let (_failed_checks, notification) = sync_problem_states_from_output_with_path(
        &config_path,
        "FR24 Link: down ... failed!\nReceiver: down ... failed!",
    )
    .unwrap();

    server_thread.join().unwrap();

    let updated = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            ["problems_link"],
            |row| row.get::<_, String>(0),
        )
        .unwrap();

    assert_eq!(updated, "true");
    assert!(notification.is_some());
}
