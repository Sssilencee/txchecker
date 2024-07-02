use anyhow::Context;
use app::domain::{height::Height, state::State, transfer::IncomingTransferParsed};
use const_format::concatcp;
use lazy_channel::mpsc::receiver::LazyUnboundedReceiver;
use log::{debug, error};
use tokio::{select, sync::mpsc::{unbounded_channel, Sender, UnboundedReceiver, UnboundedSender}, task};
use tokio_util::sync::CancellationToken;

use crate::{data::block::BlockRepo, domain::slot::{Slot, SlotTx, SLOT_CONFIRMATION_LAG}};

use super::mappers::TryIntoParsed;

pub struct TransfersServiceActor<B> {
    state_rx: UnboundedReceiver<State>,
    slot_tx: Sender<SlotTx>,
    slot_rx: LazyUnboundedReceiver<Slot>,
    transfers_tx: UnboundedSender<IncomingTransferParsed>,
    height_tx: UnboundedSender<Height>,
    block_repo: B,
}

impl<B: BlockRepo> TransfersServiceActor<B> {
    pub fn new(
        state_rx: UnboundedReceiver<State>,
        slot_tx: Sender<SlotTx>,
        transfers_tx: UnboundedSender<IncomingTransferParsed>,
        height_tx: UnboundedSender<Height>,
        block_repo: B,
    ) -> Self {
        Self { state_rx, slot_tx, transfers_tx, height_tx, block_repo, slot_rx: Default::default() }
    }

    pub async fn start(mut self, token: CancellationToken) {
        const FN_CTX: &str = "TransferServiceActor::start()";

        loop {
            select! {
                Some(state) = self.state_rx.recv() => if let Err(e) = self.process_state(state).await {
                    error!("err self.process_state() in {}: {:#?}", FN_CTX, e);
                },

                Some(slot) = self.slot_rx.recv() => if let Err(e) = self.process_slot(slot).await {
                    error!("err self.process_slot() in {}: {:#?}", FN_CTX, e);
                },

                _ = token.cancelled() => return,

                _ = task::yield_now() => continue,
            }
        }
    }

    async fn process_state(&mut self, state: State) -> anyhow::Result<()> {
        match state {
            State::Running if self.slot_rx.is_closed() => {
                let (tx, rx) = unbounded_channel();

                self.slot_rx.init(rx);
                self.slot_tx.send(tx)
                    .await
                    .context("err slot_tx.send() in process_state()")?;
            },

            State::Stopping => self.slot_rx.close(),

            _ => (),
        }

        Ok(())
    }

    async fn process_slot(&mut self, slot: Slot) -> anyhow::Result<()> {
        const FN_CTX: &str = "process_slot()";

        debug!("[{}] - new slot: {}", FN_CTX, slot);

        let transfers = self.block_repo.get_block(slot.saturating_sub(SLOT_CONFIRMATION_LAG))
            .await
            .context(concatcp!("err block_repo.get_block() in ", FN_CTX))?;

        for transfer in transfers.into_iter() {
            let transfer_parsed = transfer.try_into_parsed()
                .context(concatcp!("err transfer.try_into_parsed() in ", FN_CTX))?;

            self.transfers_tx.send(transfer_parsed)
                .context(concatcp!("err transfers_tx.send() in ", FN_CTX))?;
        }

        self.height_tx.send(slot)
            .context(concatcp!("err height_tx.send() in ", FN_CTX))?;

        Ok(())
    }
}