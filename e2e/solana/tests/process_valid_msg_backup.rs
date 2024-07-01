use std::{mem, process::{Command, Stdio}, time::Duration};

use amqprs::{channel::{BasicPublishArguments, QueuePurgeArguments}, BasicProperties};
use clap::Parser;
use config::args::SOLANA_CONFIG_NAME_DEV;
use leveldb::{kv::KV, options::WriteOptions};
use log::info;
use solana::{args::Args, killer::Killer};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer};
use storage::height::models::HeightKey;
use tokio::time::sleep;

/// # cargo run examle:
/// ```bash
/// RUST_LOG=info cargo test --test process_valid_msg_backup -- \
///     --rabbitmq /opt/homebrew/opt/rabbitmq/sbin/rabbitmq-server \
///     --tx-checker ../../target/debug/tx-checker \
///     --solana-test-validator ~/.local/share/solana/install/active_release/bin/solana-test-validator
/// ```
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    const ID: &str = "id";
    const AMOUNT: u64 = LAMPORTS_PER_SOL / 2;

    env_logger::init();

    let env_path = "../../.env";
    dotenv::from_path(env_path)?;

    let network_config_path = "../../".to_owned() + SOLANA_CONFIG_NAME_DEV;
    let network_config = config::network::load(&network_config_path)?;

    let queues_config = network_config.queues;
    let transfer_queue_name = queues_config.input_queue_name;
    let result_queue_name = queues_config.output_queue_name;

    let rabbitmq_config = config::rabbitmq::load()?;

    let mut killer = Killer::default();

    let args = Args::parse();

    {
        let handle = Command::new(args.rabbitmq)
            .stdout(Stdio::null())
            .spawn()?;
        killer.add_process(handle.id());

        let handle = Command::new(args.solana_test_validator)
            .stdout(Stdio::null())
            .spawn()?;
        killer.add_process(handle.id());

        sleep(Duration::from_secs(10)).await;
    }

    let rabbitmq_connection = rabbitmqlib::connect(&rabbitmq_config).await?;
    let channel = rabbitmq_connection.open_channel(None).await?;

    let rpc_client = RpcClient::new(network_config.rpc.http_endpoint_url);

    let height_connection = leveldblib::connect(&network_config.db.height_path)?;

    {
        let args = QueuePurgeArguments::new(&transfer_queue_name);
        channel.queue_purge(args).await?;

        let args = QueuePurgeArguments::new(&result_queue_name);
        channel.queue_purge(args).await?;
    }

    let receiver_kp = Keypair::new();
    let sender_kp = Keypair::new();

    let receiver_pk = receiver_kp.pubkey();
    let sender_pk = sender_kp.pubkey();

    {
        let properties = BasicProperties::default()
            .with_persistence(true)
            .finish();
        let args = BasicPublishArguments::new("", &transfer_queue_name);
        let msg = format!(r#"{{"id":"{}","address":"{}","amount":{}}}"#, ID, receiver_pk.to_string(), AMOUNT);
        let payload = msg.as_bytes().to_vec();
        channel.basic_publish(properties, payload, args).await?;
    }

    let (signature, slot) = {
        let slot = rpc_client.get_slot().await?;

        let signature = solana::rpc::send_transfer(
            &rpc_client,
            &receiver_pk, &sender_kp, &sender_pk,
            AMOUNT,
        ).await?;

        info!("signature - {}; slot: - {}", signature.to_string(), slot);

        (signature, slot)
    };

    {
        let options = WriteOptions::new();
        height_connection.delete(options, HeightKey::default())?;

        let options = WriteOptions::new();
        height_connection.put(options, HeightKey::default(), &slot.to_le_bytes())?;

        mem::drop(height_connection);
    }

    loop {
        let current_slot = rpc_client.get_slot_with_commitment(CommitmentConfig::processed()).await?;
        info!("current_slot - {}", current_slot);

        if current_slot - 200 > slot {
            break;
        }

        sleep(Duration::from_secs(1)).await;
    }

    let handle = Command::new(args.tx_checker)
        .arg("--dev")
        .arg("--env")
        .arg(env_path)
        .arg("--solana-config")
        .arg(network_config_path)
        .spawn()?;
    killer.add_process(handle.id());

    solana::queue::confirm_transfer(&channel, signature, result_queue_name, ID).await?;

    channel.close().await?;

    Ok(())
}