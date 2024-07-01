use amqprs::channel::{BasicAckArguments, BasicConsumeArguments, Channel};
use queue::producer::messages::ResultMsg;
use solana_sdk::signature::Signature;

pub async fn confirm_transfer(
    channel: &Channel,
    signature: Signature,
    queue_name: String,
    id: &str,
) -> anyhow::Result<()> {
    let args = BasicConsumeArguments::default()
        .queue(queue_name)
        .finish();
    let (_, mut rx) = channel.basic_consume_rx(args).await?;

    if let Some(msg) = rx.recv().await {
        let (content, deliver) = (msg.content.unwrap(), msg.deliver.unwrap());
        let result: ResultMsg = sonic_rs::from_slice(&content)?;

        let signature = signature.to_string();
        assert!(result.signatures.unwrap()
            .into_iter()
            .any(|s| s == signature)
        );
        assert_eq!(id.to_string(), result.id);

        channel.basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false)).await?;
    }

    Ok(())
}