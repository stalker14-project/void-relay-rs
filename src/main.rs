use std::{path::PathBuf, str::FromStr};
use env_logger::Env;
use log::{error, info, warn};
use void_relay::config::Config;

static DEV_CONFIG_PATH: &str = "appsettings.dev.json";
static BASE_CONFIG_PATH: &str = "appsettings.json";

#[tokio::main]
pub async fn main() {
    let env=  Env::new()
                .filter_or("MY_LOG_LEVEL", "debug")
                .filter_or("RUST_LOG", "void_relay")
                .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    // load dev config if present and debug_assertions is turned on
    let config = if cfg!(debug_assertions) && PathBuf::from_str(DEV_CONFIG_PATH).unwrap().exists() {
        info!("Reading dev configuration.");
        let config = Config::from_str(DEV_CONFIG_PATH);
        match config {
            Ok(c) => c,
            Err(e) => {
                warn!("Error reading dev configuration: {e}. Falling back to base.");
                read_base_cfg(BASE_CONFIG_PATH)
            }
        }
    } else {
        info!("Reading base configuration");
        read_base_cfg(BASE_CONFIG_PATH)
    };

    if let Err(e) = void_relay::run(config).await {
        error!("{e}");
    }
}

fn read_base_cfg(path: &str) -> Config {
    match Config::from_str(path) {
        Ok(c) => c,
        Err(e) => {
            error!("Error reading configuration: {e}");
            std::process::exit(1);
        }
    }
}