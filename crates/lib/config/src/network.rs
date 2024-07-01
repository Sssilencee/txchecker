use std::fs;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct NetworkConfig {
    pub queues: QueuesConfig,
    pub rpc: RpcConfig,
    pub db: DbConfig,
}

#[derive(Deserialize)]
pub struct QueuesConfig {
    pub input_queue_name: String,
    pub output_queue_name: String,
}

#[derive(Deserialize)]
pub struct RpcConfig {
    pub http_endpoint_url: String,
    pub ws_endpoint_url: String,
}

#[derive(Deserialize)]
pub struct DbConfig {
    pub payments_path: String,
    pub height_path: String,
}

pub fn load(path: &str) -> anyhow::Result<NetworkConfig> {
    let content = fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}