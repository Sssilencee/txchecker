use std::time::Duration;

use anyhow::Context;
use const_format::concatcp;
use fastwebsockets::{FragmentCollector, Frame, Payload};
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use log::error;
use tokio::{select, sync::mpsc::{channel, Receiver, Sender, UnboundedSender}, time::sleep};
use tokio_util::sync::CancellationToken;

use crate::{data::{req::RpcReq, res::{RpcNotification, RpcRes}}, domain::slot::{Slot, SlotTx}};

use super::res::{SlotNotification, SubscriptionId};

type Fc = FragmentCollector<TokioIo<Upgraded>>;

pub struct SlotActor {
    slot_rx: Receiver<SlotTx>,
    initial_slot: Option<Slot>,
    fc: Fc,
}

impl SlotActor {
    pub fn new(fc: Fc, initial_slot: Option<Slot>) -> (Self, Sender<SlotTx>) {
        let (tx, slot_rx) = channel(1);
        (Self { slot_rx, initial_slot, fc }, tx)
    }

    pub async fn start(mut self, token: CancellationToken) {
        loop {
            select! {
                Some(tx) = self.slot_rx.recv() => if let Err(e) = self.serve_slot_subscription(tx).await {
                    error!("err self.serve_slot_subscription() in SlotActor::start(): {:#?}", e);
                },

                _ = token.cancelled() => return,

                _ = sleep(Duration::from_millis(0)) => continue,
            }
        }
    }

    async fn slot_subscribe(&mut self) -> anyhow::Result<Slot> {
        const FN_CTX: &str = "slot_subscribe()";

        let req: RpcReq<()> = RpcReq::new_slot_subscribe();
        let payload = sonic_rs::to_vec(&req)
            .context(concatcp!("err sonic_rs::to_vec() in ", FN_CTX))?;

        self.fc.write_frame(Frame::text(Payload::Borrowed(&payload)))
            .await
            .context(concatcp!("err fc.write_frame() in", FN_CTX))?;

        let frame = self.fc.read_frame()
            .await
            .context(concatcp!("err fc.read_frame() in", FN_CTX))?;
        let res: RpcRes<Slot> = sonic_rs::from_slice(&frame.payload)
            .context(concatcp!("err sonic_rs::from_slice() in", FN_CTX))?;

        Ok(res.result)
    }

    async fn slot_unsubscribe(&mut self, subscription_id: SubscriptionId) -> anyhow::Result<()> {
        const FN_CTX: &str = "slot_unsubscribe()";

        let req = RpcReq::new_slot_unsubscribe(subscription_id);
        let payload = sonic_rs::to_vec(&req)
            .context(concatcp!("err sonic_rs::to_vec() in ", FN_CTX))?;

        self.fc.write_frame(Frame::text(Payload::Borrowed(&payload)))
            .await
            .context(concatcp!("err fc.write_frame() in ", FN_CTX))?;

        Ok(())
    }

    async fn serve_slot_subscription(&mut self, tx: UnboundedSender<Slot>) -> anyhow::Result<()> {
        const FN_CTX: &str = "serve_slot_subscription()";

        let subscription_id = self.slot_subscribe()
            .await
            .context(concatcp!("err self.slot_subscribe() in ", FN_CTX))?;

        loop {
            if tx.is_closed() {
                self.slot_unsubscribe(subscription_id)
                    .await
                    .context(concatcp!("err self.slot_unsubscribe in ", FN_CTX))?;

                break;
            }

            let frame = self.fc.read_frame()
                .await
                .context(concatcp!("err fc.read_frame() in ", FN_CTX))?;
            let notification: RpcNotification<SlotNotification> = sonic_rs::from_slice(&frame.payload)
                .context(concatcp!("err sonic_rs::from_slice() in ", FN_CTX))?;

            let slot = notification.params.result.slot;

            if let Some(initial_slot) = self.initial_slot {
                for slot in initial_slot..slot {
                    tx.send(slot)
                        .context(concatcp!("err tx.send(initial_slot) in ", FN_CTX))?;
                }
                self.initial_slot = None;
            }

            tx.send(slot)
                .context(concatcp!("err tx.send(slot) in ", FN_CTX))?;
        }

        Ok(())
    }
}