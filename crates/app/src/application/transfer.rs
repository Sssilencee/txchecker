use anyhow::Context;
use const_format::concatcp;
use log::error;
use queue::{consumer::messages::{ConsumerMsg, TransferMsg}, producer::messages::{ProducerMsg, ResultMsg}};
use tokio::{select, sync::mpsc::{UnboundedReceiver, UnboundedSender}, task};
use tokio_util::sync::CancellationToken;

use crate::{application::mappers::IntoIncomingPayment, domain::{payment::{IncomingPayment, ProcessedPaymentMeta}, pubkey::Pubkey, state::State}};

pub trait ParserPort {
    fn to_pubkey(&self, address: &String) -> anyhow::Result<Pubkey>;
}

pub struct TransferActor<P> {
    messages_rx: UnboundedReceiver<ConsumerMsg<TransferMsg>>,
    payments_tx: UnboundedSender<IncomingPayment>,
    state_tx: UnboundedSender<State>,
    processed_rx: UnboundedReceiver<ProcessedPaymentMeta>,
    producer_tx: UnboundedSender<ProducerMsg<ResultMsg>>,
    parser: P,
}

impl<P: ParserPort> TransferActor<P> {
    pub fn new(
        messages_rx: UnboundedReceiver<ConsumerMsg<TransferMsg>>,
        payments_tx: UnboundedSender<IncomingPayment>,
        state_tx: UnboundedSender<State>,
        processed_rx: UnboundedReceiver<ProcessedPaymentMeta>,
        producer_tx: UnboundedSender<ProducerMsg<ResultMsg>>,
        parser: P
    ) -> Self {
        Self { messages_rx, payments_tx, state_tx, processed_rx, producer_tx, parser }
    }

    pub async fn start(mut self, token: CancellationToken) {
        const FN_CTX: &str = "TransferActor::start()";

        loop {
            select! {
                Some(consumer_msg) = self.messages_rx.recv() => if let Err(e) = self.process_message(consumer_msg).await {
                    error!("err process_message() in {}: {:#?}", FN_CTX, e);
                },

                Some(meta) = self.processed_rx.recv() => if let Err(e) = self.process_payment(meta) {
                    error!("err process_result() in {}: {:#?}", FN_CTX, e);
                },

                _ = token.cancelled() => return,

                _ = task::yield_now() => continue,
            }
        }
    }

    async fn process_message(&self, consumer_msg: ConsumerMsg<TransferMsg>) -> anyhow::Result<()> {
        const FN_CTX: &str = "process_message()";

        let pubkey = self.parser.to_pubkey(&consumer_msg.msg.address)
            .context(concatcp!("err parser.to_pubkey() in ", FN_CTX))?;

        self.payments_tx.send(consumer_msg.into_domain(pubkey))
            .context(concatcp!("err payments_tx.send() in ", FN_CTX))?;

        self.state_tx.send(State::Running)
            .context(concatcp!("err state_tx.send() in ", FN_CTX))?;

        Ok(())
    }

    fn process_payment(&self, meta: ProcessedPaymentMeta) -> anyhow::Result<()> {
        const FN_CTX: &str = "process_payment()";

        let last = meta.last();
        if last {
            self.state_tx.send(State::Stopping)
                .context(concatcp!("err state_tx.send() in ", FN_CTX))?;
        }

        self.producer_tx.send(meta.payment().into())
            .context(concatcp!("err producer_tx.send() in ", FN_CTX))?;

        Ok(())
    }
}