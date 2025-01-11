use std::str::FromStr;
use env_logger::Env;
use log::error;
use void_relay::config::Config;

#[tokio::main]
pub async fn main() {
    let env=  Env::new()
                .filter_or("MY_LOG_LEVEL", "debug")
                .filter_or("RUST_LOG", "void_relay")
                .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let config = match Config::from_str("appsettings.json") {
        Ok(c) => c,
        Err(e) => {
            error!("Unable to parse configuration: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = void_relay::run(config).await {
        error!("{e}");
    }
}