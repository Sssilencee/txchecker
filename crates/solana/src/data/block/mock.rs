use app::domain::transfer::{IncomingTransfer, TransferData};
use tokio::sync::Mutex;

use crate::domain::slot::Slot;

use super::BlockRepo;

pub struct BlockServiceMock {
    meta: Mutex<Option<TransferMetadata>>,
}

struct TransferMetadata {
    address: String,
    amount: u64,
    signature: String,
}

impl TransferMetadata {
    pub fn new(address: String, amount: u64, signature: String) -> Self {
        Self { address, amount, signature }
    }
}

impl BlockServiceMock {
    pub fn new(address: String, amount: u64, signature: String) -> Self {
        let meta = Mutex::new(Some(TransferMetadata::new(address, amount, signature)));

        Self { meta }
    }
}

impl BlockRepo for BlockServiceMock {
    async fn get_block(&self, _slot: Slot) -> anyhow::Result<Vec<IncomingTransfer>> {
        let address = self.meta
            .lock()
            .await
            .take();

        Ok(match address {
            Some(m) => vec![
                IncomingTransfer::new(
                    TransferData::new(m.address, m.amount),
                    vec![m.signature],
                )
            ],
            None => Vec::new(),
        })
    }
}