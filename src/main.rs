use anyhow::Result;
use app::application::transfer::TransferActor;
use config::args;
use fastwebsocketslib;
use log::error;
use queue::{consumer::ConsumerActor, producer::ProducerActor};
use storage::{height::HeightActor, payments::PaymentsActor};
use tokio::{signal, sync::mpsc::unbounded_channel};
use hyperlib;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<()> {
    let args = args::load();

    env_loggerlib::init(args.dev);

    dotenv::from_path(args.env)?;

    let rabbitmq_config = config::rabbitmq::load()?;

    let token = CancellationToken::new();

    #[cfg(feature = "solana")]
    {
        use solana::{
            data::{block::BlockService, slot::SlotActor},
            service::{transfers::TransfersServiceActor, parser::Parser},
        };

        let network_config = config::network::load(&args.solana_config)?;

        let queues_config = network_config.queues;
        let rpc_config = network_config.rpc;
        let db_config = network_config.db;

        let transfer_queue_name = queues_config.input_queue_name;
        let result_queue_name = queues_config.output_queue_name;

        let consumer_connection = rabbitmqlib::connect(&rabbitmq_config).await?;
        let (consumer_actor, messages_rx) = ConsumerActor::new(consumer_connection)?;
        tokio::spawn(consumer_actor.start(transfer_queue_name.clone(), token.clone()));

        let producer_connection = rabbitmqlib::connect(&rabbitmq_config).await?;
        let (producer_actor, producer_tx) = ProducerActor::new(producer_connection, result_queue_name.clone())?;
        tokio::spawn(producer_actor.start(token.clone()));

        let height_connection = leveldblib::connect(&db_config.height_path)?;
        let (height_actor, height_tx) = HeightActor::new(height_connection);
        let height = height_actor.get_height()?;
        tokio::spawn(height_actor.start(token.clone()));

        let fc = fastwebsocketslib::connect(&rpc_config.ws_endpoint_url).await?;
        let (slot_actor, slot_tx) = SlotActor::new(fc, height);
        tokio::spawn(slot_actor.start(token.clone()));

        let payments_connection = leveldblib::connect(&db_config.payments_path)?;
        let (payments_tx, payments_rx) = unbounded_channel();
        let (transfers_tx, transfers_rx) = unbounded_channel();
        let (processed_tx, processed_rx) = unbounded_channel();
        let payments_actor = PaymentsActor::new(payments_rx, transfers_rx, processed_tx, payments_connection);
        tokio::spawn(payments_actor.start(token.clone()));

        let (state_tx, state_rx) = unbounded_channel();
        let block_service = BlockService::new(hyperlib::connect(), rpc_config.http_endpoint_url);
        let transfer_service_actor = TransfersServiceActor::new(
            state_rx, slot_tx,
            transfers_tx, height_tx,
            block_service,
        );
        tokio::spawn(transfer_service_actor.start(token.clone()));

        let parser = Parser;
        let transfer_actor = TransferActor::new(
            messages_rx, payments_tx,
            state_tx, processed_rx,
            producer_tx,
            parser,
        );
        transfer_actor.start(token.clone()).await;
    }

    match signal::ctrl_c().await {
        Ok(_) => token.cancel(),
        Err(e) => {
            error!("err signal::ctrl_c() in main(): {}", e);
            token.cancel();
        },
    }

    Ok(())
}
