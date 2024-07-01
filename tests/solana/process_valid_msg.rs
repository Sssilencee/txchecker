#[cfg(test)]
mod tests {
    use amqprs::{channel::{BasicAckArguments, BasicConsumeArguments, BasicPublishArguments}, BasicProperties};
    use app::application::transfer::TransferActor;
    use queue::{consumer::ConsumerActor, producer::{messages::ResultMsg, ProducerActor}};
    use solana::{data::{block::mock::BlockServiceMock, slot::slot_mock::SlotActorMock}, service::{transfers::TransfersServiceActor, parser::Parser}};
    use storage::{height::HeightActor, payments::PaymentsActor};
    use tokio::sync::mpsc::unbounded_channel;
    use tokio_util::sync::CancellationToken;

    #[tokio::test]
    async fn process_valid_msg() -> Result<(), anyhow::Error> {
        const ID: &str = "id";
        const ADDRESS: &str = "address";
        const AMOUNT: u64 = 1000;
        const SIGNATURE: &str = "signature";

        let args = config::args::load_default();

        dotenv::from_path(args.env)?;

        let rabbitmq_config = config::rabbitmq::load()?;
        let network_config = config::network::load(&args.solana_config)?;

        let queues_config = network_config.queues;
        let db_config = network_config.db;

        let transfer_queue_name = queues_config.input_queue_name;
        let result_queue_name = queues_config.output_queue_name;

        let token = CancellationToken::new();

        let consumer_connection = rabbitmqlib::connect(&rabbitmq_config).await?;
        let (consumer_actor, messages_rx) = ConsumerActor::new(consumer_connection)?;
        tokio::spawn(consumer_actor.start(transfer_queue_name.clone(), token.clone()));

        let producer_connection = rabbitmqlib::connect(&rabbitmq_config).await?;
        let (producer_actor, producer_tx) = ProducerActor::new(producer_connection, result_queue_name.clone())?;
        tokio::spawn(producer_actor.start(token.clone()));

        let height_connection = leveldblib::connect(&db_config.height_path)?;
        let (height_actor, height_tx) = HeightActor::new(height_connection);
        tokio::spawn(height_actor.start(token.clone()));

        let (slot_actor, slot_tx) = SlotActorMock::new();
        tokio::spawn(slot_actor.start());

        let payments_connection = leveldblib::connect(&db_config.payments_path)?;
        let (payments_tx, payments_rx) = unbounded_channel();
        let (transfers_tx, transfers_rx) = unbounded_channel();
        let (processed_tx, processed_rx) = unbounded_channel();
        let payments_actor = PaymentsActor::new(payments_rx, transfers_rx, processed_tx, payments_connection);
        tokio::spawn(payments_actor.start(token.clone()));

        let (state_tx, state_rx) = unbounded_channel();
        let block_service = BlockServiceMock::new(ADDRESS.into(), AMOUNT, SIGNATURE.into());
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
        tokio::spawn(transfer_actor.start(token.clone()));

        {
            let connection = rabbitmqlib::connect(&rabbitmq_config).await?;
            let channel = connection.open_channel(None).await?;

            let properties = BasicProperties::default()
                .with_persistence(true)
                .finish();
            let args = BasicPublishArguments::new("", &transfer_queue_name);
            let msg = format!(r#"{{"id":"{}","address":"{}","amount":{}}}"#, ID, ADDRESS, AMOUNT);
            let payload = msg.as_bytes().to_vec();
            channel.basic_publish(properties, payload, args).await?;

            let args = BasicConsumeArguments::default()
                .queue(result_queue_name)
                .finish();
            let (_, mut rx) = channel.basic_consume_rx(args).await?;

            if let Some(msg) = rx.recv().await {
                let (content, deliver) = (msg.content.unwrap(), msg.deliver.unwrap());
                let result: ResultMsg = sonic_rs::from_slice(&content)?;

                let signature = SIGNATURE.to_string();
                assert!(result.signatures.unwrap()
                    .into_iter()
                    .any(|s| s == signature)
                );
                assert_eq!(ID.to_string(), result.id);

                channel.basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false)).await?;
            }

            channel.close().await?;
        }

        Ok(())
    }
}
