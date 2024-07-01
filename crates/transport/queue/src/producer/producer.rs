use std::{fmt::Debug, time::Duration};

use amqprs::{channel::{BasicAckArguments, BasicPublishArguments, Channel}, connection::Connection, BasicProperties};
use anyhow::Context;
use const_format::concatcp;
use log::{debug, error, info};
use serde::Serialize;
use tokio::{select, sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender}, time::sleep};
use tokio_util::sync::CancellationToken;

use crate::consumer::messages::DeliveryTag;

use super::messages::ProducerMsg;

pub struct ProducerActor<T> {
    connection: Connection,
    queue_name: String,
    messages_rx: UnboundedReceiver<ProducerMsg<T>>,
}

impl<T: Serialize + Debug> ProducerActor<T> {
    pub fn new(
        connection: Connection,
        queue_name: String,
    ) -> anyhow::Result<(Self, UnboundedSender<ProducerMsg<T>>)> {
        let (tx, messages_rx) = unbounded_channel();

        Ok((Self { connection, queue_name, messages_rx }, tx))
    }

    pub async fn start(mut self, token: CancellationToken) -> anyhow::Result<()> {
        let channel = self.connection.open_channel(None).await?;

        loop {
            select! {
                Some(msg) = self.messages_rx.recv() => if let Err(e) = self.process_message(&channel, msg).await {
                    error!("err process_message() in ProducerActor::start(): {:#?}", e);
                },

                _ = token.cancelled() => return self.stop(channel).await,

                _ = sleep(Duration::from_millis(0)) => continue,
            }
        }
    }

    async fn process_message(&self, channel: &Channel, producer_msg: ProducerMsg<T>) -> anyhow::Result<()> {
        const FN_CTX: &str = "process_message()";

        let payload = sonic_rs::to_vec(&producer_msg.msg)
            .context(concatcp!("err sonic_rs::to_vec() in ", FN_CTX))?;

        let args = BasicPublishArguments::new("", &self.queue_name);
        let properties = BasicProperties::default()
            .with_persistence(true)
            .finish();

        channel.basic_publish(properties, payload, args)
            .await
            .context(concatcp!("err channel.basic_publish() in ", FN_CTX))?;

        info!("[{}; queue: {}] - send msg: {:?}", FN_CTX, self.queue_name, producer_msg);

        self.commit_message(channel, producer_msg.tag)
            .await
            .context(concatcp!("err self.commit_message() in ", FN_CTX))?;

        debug!("[{}; queue: {}] - commit msg: {:?}", FN_CTX, self.queue_name, producer_msg);

        Ok(())
    }

    async fn commit_message(&self, channel: &Channel, tag: DeliveryTag) -> anyhow::Result<()> {
        channel.basic_ack(BasicAckArguments::new(tag, false))
            .await
            .context("err channel.basic_ack() in commit_message()")?;
        Ok(())
    }

    async fn stop(self, channel: Channel) -> anyhow::Result<()> {
        channel.close().await?;
        self.connection.close().await?;
        Ok(())
    }
}