use std::fmt::Debug;

use amqprs::{channel::{BasicConsumeArguments, Channel, ConsumerMessage}, connection::Connection};
use anyhow::{bail, Context};
use const_format::concatcp;
use log::{error, info};
use serde::de::DeserializeOwned;
use tokio::{select, sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender}, task};
use tokio_util::sync::CancellationToken;

use super::messages::ConsumerMsg;

pub struct ConsumerActor<T> {
    connection: Connection,
    messages_tx: UnboundedSender<ConsumerMsg<T>>,
}

impl<T> ConsumerActor<T>
where
    T: DeserializeOwned + Sync + Send + 'static + Debug,
{
    pub fn new(
        connection: Connection,
    ) -> anyhow::Result<(Self, UnboundedReceiver<ConsumerMsg<T>>)> {
        let (messages_tx, rx) = unbounded_channel();

        Ok((Self { connection, messages_tx }, rx))
    }

    pub async fn start(self, queue_name: String, token: CancellationToken) -> anyhow::Result<()> {
        const FN_CTX: &str = "ConsumerActor::start()";

        let channel = self.connection.open_channel(None)
            .await
            .context(concatcp!("err connection.open_channel() in ", FN_CTX))?;

        let args = BasicConsumeArguments::default()
            .queue(queue_name)
            .finish();
        let (_, mut consumer) = channel.basic_consume_rx(args)
            .await
            .context(concatcp!("err channel.basic_consume_rx() in ", FN_CTX))?;

        loop {
            select! {
                Some(msg) = consumer.recv() => if let Err(e) = self.process_message(msg) {
                    error!("err process_message() in {}: {:#?}", FN_CTX, e);
                },

                _ = token.cancelled() => return self.stop(channel).await,

                _ = task::yield_now() => continue,
            }
        }
    }

    fn process_message(&self, msg: ConsumerMessage) -> anyhow::Result<()> {
        const FN_CTX: &str = "process_message()";

        match (msg.content, msg.deliver) {
            (Some(p), Some(d)) => {
                let msg = sonic_rs::from_slice(&p)
                    .context(concatcp!("err sonic_rs::from_slice() in ", FN_CTX))?;

                info!("{} - new msg: {:?}", FN_CTX, msg);

                self.messages_tx.send(ConsumerMsg::new(msg, d.delivery_tag()))
                    .context(concatcp!("err messages_tx.send() in ", FN_CTX))?;
            },
            _ => bail!("err 'msg is empty' in {}", FN_CTX),
        }

        Ok(())
    }

    async fn stop(self, channel: Channel) -> anyhow::Result<()> {
        channel.close().await?;
        self.connection.close().await?;
        Ok(())
    }
}