use std::time::Duration;

use tokio::{sync::mpsc::{channel, Receiver, Sender}, time::sleep};

use crate::domain::slot::SlotTx;

pub struct SlotActorMock {
    slot_rx: Receiver<SlotTx>,
}

impl SlotActorMock {
    pub fn new() -> (Self, Sender<SlotTx>) {
        let (tx, slot_rx) = channel(1);
        (Self { slot_rx }, tx)
    }

    pub async fn start(mut self) -> anyhow::Result<()> {
        let mut slot_number = 0;

        let slot_tx = self.slot_rx.recv()
            .await
            .unwrap();

        loop {
            slot_tx.send(slot_number)?;
            slot_number += 1;

            sleep(Duration::from_millis(400)).await;
        }
    }
}