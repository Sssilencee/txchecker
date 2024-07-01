use serde::Deserialize;

#[derive(Deserialize)]
pub struct RabbitMqConfig {
    #[serde(default="default_host")]
    pub host: String,

    #[serde(default="default_port")]
    pub port: u16,

    #[serde(default="default_username")]
    pub username: String,

    #[serde(default="default_password")]
    pub password: String,
}

#[inline]
fn default_host() -> String {
    "localhost".into()
}

#[inline]
fn default_port() -> u16 {
    5672
}

#[inline]
fn default_username() -> String {
    "admin".into()
}

#[inline]
fn default_password() -> String {
    "admin".into()
}

pub fn load() -> anyhow::Result<RabbitMqConfig> {
    Ok(envy::prefixed("RABBITMQ_").from_env()?)
}