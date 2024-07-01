use app::domain::transfer::{IncomingTransfer, IncomingTransferParsed, TransferDataParsed};

use crate::service::parser::to_pubkey;

pub trait TryIntoParsed {
    fn try_into_parsed(self) -> anyhow::Result<IncomingTransferParsed>;
}

impl TryIntoParsed for IncomingTransfer {
    fn try_into_parsed(self) -> anyhow::Result<IncomingTransferParsed> {
        let (transfer_data, signatures) = self.expose();
        let amount = transfer_data.amount();
        let pubkey = to_pubkey(&transfer_data.address())?;

        let transfer_data = TransferDataParsed::new(pubkey, amount);
        Ok(IncomingTransferParsed::new(transfer_data, signatures))
    }
}