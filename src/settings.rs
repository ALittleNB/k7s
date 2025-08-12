use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{fs, path::Path};
use tracing::{error, info};

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub auth: AuthSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthSettings {
    pub password: String,
}

impl Settings {
    fn any_config_file_exists() -> bool {
        [
            Path::new("k7s"),
            Path::new("k7s.yaml"),
            Path::new("config/k7s"),
            Path::new("config/k7s.yaml"),
        ]
        .iter()
        .any(|p| p.exists())
    }

    fn create_example_config_or_exit() -> ! {
        let example = r#"# k7s example configuration
server:
  host: "127.0.0.1"
  port: 18080
auth:
  password: "changeme"  # please change to a strong password
"#;
        match fs::write("k7s.yaml", example) {
            Ok(_) => {
                error!(
                    "no configuration file found; generated example at 'k7s.yaml'. please edit it and restart"
                );
            }
            Err(e) => {
                error!("failed to write example config 'k7s.yaml': {}", e);
            }
        }
        std::process::exit(1);
    }

    pub fn load_or_exit() -> Self {
        if !Self::any_config_file_exists() {
            Self::create_example_config_or_exit();
        }

        // Load from files using the `config` crate. We intentionally avoid env vars here.
        let builder = config::Config::builder()
            // Common names and locations
            .add_source(config::File::with_name("k7s").required(false))
            .add_source(config::File::with_name("k7s.yaml").required(false))
            .add_source(config::File::with_name("config/k7s").required(false))
            .add_source(config::File::with_name("config/k7s.yaml").required(false))
            // Provide reasonable defaults so the binary can run even without a config file.
            .set_default("server.host", "0.0.0.0").unwrap()
            .set_default("server.port", 8080).unwrap()
            .set_default("auth.password", "changeme").unwrap();

        let conf = match builder.build() {
            Ok(c) => c,
            Err(e) => {
                error!("failed to load configuration: {}", e);
                std::process::exit(1);
            }
        };

        match conf.try_deserialize() {
            Ok(s) => {
                info!("configuration loaded");
                s
            }
            Err(e) => {
                error!("failed to parse configuration: {}", e);
                std::process::exit(1);
            }
        }
    }
}

pub static SETTINGS: Lazy<Settings> = Lazy::new(Settings::load_or_exit);


