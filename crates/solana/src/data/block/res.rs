use app::domain::transfer::{IncomingTransfer, TransferData};
use itertools::izip;
use sonic_rs::Deserialize;

#[derive(Deserialize)]
pub struct GetBlockRes {
    transactions: Vec<GetTransactionRes>,
}

impl Into<Vec<IncomingTransfer>> for GetBlockRes {
    fn into(self) -> Vec<IncomingTransfer> {
        let mut transfers = Vec::new();

        for tx in self.transactions {
            let tx_data = tx.transaction;
            let (pre, post) = (tx.meta.pre_balances, tx.meta.post_balances);

            let tx_transfers = izip!(tx_data.message.account_keys, pre, post)
                .filter_map(|(addr, pre, post)| match pre < post {
                    true => Some(IncomingTransfer::new(
                        TransferData::new(addr, post - pre),
                        tx_data.signatures.clone(),
                    )),
                    false => None,
                });

            transfers.extend(tx_transfers);
        }

        transfers
    }
}

#[derive(Deserialize)]
pub struct GetTransactionRes {
    meta: Meta,
    transaction: Transaction,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    post_balances: Vec<u64>,
    pre_balances: Vec<u64>,
}

#[derive(Deserialize)]
pub struct Transaction {
    message: Message,
    signatures: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    account_keys: Vec<String>,
}