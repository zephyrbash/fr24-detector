pub const CONFIG_PATH: &str = "/etc/fr24detector.ini";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const DEFAULT_CONFIG_CONTENT: &str = "
[general]
webhook_url=

[do_not_modify_states]
problems_link=false
problems_receiver=false
problems_started=
";
