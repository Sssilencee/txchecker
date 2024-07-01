use amqprs::connection::{Connection, OpenConnectionArguments};
use config::rabbitmq::RabbitMqConfig;

pub async fn connect(config: &RabbitMqConfig) -> anyhow::Result<Connection> {
    Ok(Connection::open(&OpenConnectionArguments::new(
        &config.host, config.port,
        &config.username, &config.password,
    )).await?)
}