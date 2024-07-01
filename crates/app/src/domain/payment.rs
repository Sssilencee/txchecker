use std::mem;

use super::transfer::TransferDataParsed;

pub struct IncomingPayment {
    id: String,
    tag: u64,
    transfer_data: TransferDataParsed,
}

impl IncomingPayment {
    pub fn new(id: String, tag: u64, transfer_data: TransferDataParsed) -> Self {
        Self { id, tag, transfer_data }
    }

    #[inline]
    pub fn tag(&self) -> u64 {
        self.tag
    }

    #[inline]
    pub fn expose(self) -> (String, TransferDataParsed) {
        (self.id, self.transfer_data)
    }
}

pub struct ProcessedPaymentMeta {
    payment: ProcessedPayment,
    last: bool,
}

impl ProcessedPaymentMeta {
    pub fn new(payment: ProcessedPayment, last: bool) -> Self {
        Self { payment, last }
    }

    #[inline]
    pub fn payment(self) -> ProcessedPayment {
        self.payment
    }

    #[inline]
    pub fn last(&self) -> bool {
        self.last
    }
}

pub struct ProcessedPayment {
    id: String,
    tag: u64,
    signatures: Option<Vec<String>>,
    error: Option<()>,
}

impl ProcessedPayment {
    pub fn new(
        id: String,
        tag: u64,
        signatures: Option<Vec<String>>,
        error: Option<()>,
    ) -> Self {
        Self { id, tag, signatures, error }
    }

    #[inline]
    pub fn id(self) -> String {
        self.id
    }

    #[inline]
    pub fn tag(&self) -> u64 {
        self.tag
    }

    #[inline]
    pub fn signatures(&mut self) -> Option<Vec<String>> {
        self.signatures
            .as_mut()
            .map(|sigs| mem::take(sigs))
    }

    #[inline]
    pub fn error(&mut self) -> Option<()> {
        self.error
            .as_mut()
            .map(|e| mem::take(e))
    }
}