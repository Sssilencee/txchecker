use super::pubkey::Pubkey;

#[derive(Debug)]
pub struct TransferData {
    address: String,
    amount: u64,
}

impl TransferData {
    pub fn new(address: String, amount: u64) -> Self {
        Self { address, amount }
    }

    #[inline]
    pub fn address(self) -> String {
        self.address
    }

    #[inline]
    pub fn amount(&self) -> u64 {
        self.amount
    }
}

#[derive(Debug)]
pub struct TransferDataParsed {
    pubkey: Pubkey,
    amount: u64,
}

impl TransferDataParsed {
    pub fn new(pubkey: Pubkey, amount: u64) -> Self {
        Self { pubkey, amount }
    }

    #[inline]
    pub fn pubkey(self) -> Pubkey {
        self.pubkey
    }

    #[inline]
    pub fn amount(&self) -> u64 {
        self.amount
    }
}

#[derive(Debug)]
pub struct IncomingTransfer {
    transfer_data: TransferData,
    signatures: Vec<String>,
}

impl IncomingTransfer {
    pub fn new(transfer_data: TransferData, signatures: Vec<String>) -> Self {
        Self { transfer_data, signatures }
    }

    #[inline]
    pub fn expose(self) -> (TransferData, Vec<String>) {
        (self.transfer_data, self.signatures)
    }
}

#[derive(Debug)]
pub struct IncomingTransferParsed {
    transfer_data: TransferDataParsed,
    signatures: Vec<String>,
}

impl IncomingTransferParsed {
    pub fn new(transfer_data: TransferDataParsed, signatures: Vec<String>) -> Self {
        Self { transfer_data, signatures }
    }

    #[inline]
    pub fn expose(self) -> (TransferDataParsed, Vec<String>) {
        (self.transfer_data, self.signatures)
    }
}