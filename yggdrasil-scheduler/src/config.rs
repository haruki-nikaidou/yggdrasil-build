use clap::Parser;
use serde::{Deserialize, Serialize};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

pub(crate) fn set_tracing_subscriber(level: LogLevel) {
    let log_level = match level {
        LogLevel::Debug => tracing::Level::DEBUG,
        LogLevel::Info => tracing::Level::INFO,
        LogLevel::Warn => tracing::Level::WARN,
        LogLevel::Error => tracing::Level::ERROR,
    };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub ms: u32,
    #[serde(default = "default_log_level")]
    pub log_level: LogLevel,
    pub nats: String,
    pub postgres: String,
}

fn default_log_level() -> LogLevel {
    LogLevel::Warn
}

#[derive(Parser)]
pub struct Args {
    #[clap(short, long, default_value = "config.toml")]
    pub config_file: String,
}

pub async fn read_config_file(path: &str) -> anyhow::Result<SchedulerConfig> {
    let file = tokio::fs::read_to_string(path).await?;
    let config: SchedulerConfig = toml::from_str(&file)?;
    Ok(config)
}
