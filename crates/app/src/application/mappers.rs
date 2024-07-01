use queue::{consumer::messages::{ConsumerMsg, TransferMsg}, producer::messages::{ProducerMsg, ResultMsg}};

use crate::domain::{payment::{IncomingPayment, ProcessedPayment}, pubkey::Pubkey, transfer::TransferDataParsed};

pub trait IntoIncomingPayment {
    fn into_domain(self, pubkey: Pubkey) -> IncomingPayment;
}

impl IntoIncomingPayment for ConsumerMsg<TransferMsg> {
    fn into_domain(self, pubkey: Pubkey) -> IncomingPayment {
        let msg = self.msg;
        IncomingPayment::new(msg.id, self.tag, TransferDataParsed::new(pubkey, msg.amount))
    }
}

impl Into<ProducerMsg<ResultMsg>> for ProcessedPayment {
    fn into(mut self) -> ProducerMsg<ResultMsg> {
        let signatures = self.signatures();
        let tag = self.tag();
        let error = self.error();

        let msg = ResultMsg::new(self.id(), signatures, error);
        ProducerMsg::new(msg, tag)
    }
}