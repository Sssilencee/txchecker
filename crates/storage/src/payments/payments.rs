use std::time::Duration;

use anyhow::{bail, Context};
use app::domain::{payment::{IncomingPayment, ProcessedPayment, ProcessedPaymentMeta}, transfer::IncomingTransferParsed};
use const_format::concatcp;
use hashbrown::hash_map::EntryRef;
use leveldb::{database::Database, iterator::Iterable, kv::KV, options::{ReadOptions, WriteOptions}};
use log::error;
use tokio::{select, sync::mpsc::{UnboundedReceiver, UnboundedSender}, time::sleep};
use tokio_util::sync::CancellationToken;

use super::models::{Payment, PaymentsCache, PubkeyKey};

pub struct PaymentsActor {
    payments_rx: UnboundedReceiver<IncomingPayment>,
    transfers_rx: UnboundedReceiver<IncomingTransferParsed>,
    processed_tx: UnboundedSender<ProcessedPaymentMeta>,
    db: Database<PubkeyKey>,
    cache: PaymentsCache,
}

impl PaymentsActor {
    pub fn new(
        payments_rx: UnboundedReceiver<IncomingPayment>,
        transfers_rx: UnboundedReceiver<IncomingTransferParsed>,
        processed_tx: UnboundedSender<ProcessedPaymentMeta>,
        db: Database<PubkeyKey>,
    ) -> Self {
        Self { payments_rx, transfers_rx, processed_tx, db, cache: Default::default() }
    }

    pub async fn start(mut self, token: CancellationToken) -> anyhow::Result<()> {
        const FN_CTX: &str = "PaymentsActor::start()";

        self.load_payments()?;

        loop {
            select! {
                Some(payment) = self.payments_rx.recv() => if let Err(e) = self.process_incoming_payment(payment) {
                    error!("err self.process_incoming_payment() in {}: {:#?}", FN_CTX, e);
                },

                Some(transfer) = self.transfers_rx.recv() => if let Err(e) = self.process_incoming_transfer(transfer) {
                    error!("err self.process_incoming_transfer() in {}: {:#?}", FN_CTX, e);
                },

                _ = token.cancelled() => return Ok(()),

                _ = sleep(Duration::from_millis(0)) => continue,
            }
        }
    }

    fn process_incoming_payment(&mut self, incoming_payment: IncomingPayment) -> anyhow::Result<()>{
        let tag = incoming_payment.tag();
        let (id, transfer_data) = incoming_payment.expose();

        let amount = transfer_data.amount();
        let payment = Payment::new(id, tag, amount);
        let pubkey = transfer_data.pubkey().into();

        self.set_payment(&pubkey, &payment)
            .context("err self.set_payment() in process_incoming_payment()")?;

        self.cache.insert(pubkey, payment);

        Ok(())
    }

    fn process_incoming_transfer(&mut self, incoming_transfer: IncomingTransferParsed) -> anyhow::Result<()> {
        const FN_CTX: &str = "process_incoming_transfer()";

        let (transfer_data, signatures) = incoming_transfer.expose();
        let amount = transfer_data.amount();
        let pubkey = transfer_data.pubkey().into();

        if let EntryRef::Occupied(mut e) = self.cache.entry_ref(&pubkey) {
            let payment = e.get_mut();
            payment.signatures.extend(signatures);

            match payment.amount.checked_sub(amount) {
                Some(r) if r > 0 => payment.amount = r,
                _ => {
                    let (pubkey, p) = e.remove_entry();
                    let payment = ProcessedPayment::new(p.id, p.tag, Some(p.signatures), None);
                    let meta = ProcessedPaymentMeta::new(payment, self.cache.is_empty());

                    self.processed_tx.send(meta)
                        .context(concatcp!("err processed_tx.send() in ", FN_CTX))?;

                    self.remove_payment(&pubkey)
                        .context(concatcp!("err self.remove_payment() in ", FN_CTX))?;
                },
            }
        }

        if let Some(payment) = self.cache.get(&pubkey) {
            self.set_payment(&pubkey, payment)
                .context(concatcp!("err self.set_payment() in ", FN_CTX))?;
        }

        Ok(())
    }

    fn load_payments(&mut self) -> anyhow::Result<()> {
        const FN_CTX: &str = "load_payments()";

        let options = ReadOptions::new();

        for k in self.db.keys_iter(options) {
            let payment = self.get_payment(&k)
                .context(concatcp!("err self.get_payment() in ", FN_CTX))?;

            self.cache.insert(k, payment);
        }

        Ok(())
    }

    fn get_payment(&self, key: &PubkeyKey) -> anyhow::Result<Payment> {
        const FN_CTX: &str = "get_payment()";

        let options = ReadOptions::new();
        let b = self.db.get_bytes(options, key)
            .context(concatcp!("err db.get() in ", FN_CTX))?;

        let payment = match b {
            Some(b) => sonic_rs::from_slice(&b)
                .context(concatcp!("err sonic_rs::from_slice() in ", FN_CTX))?,

            None => bail!("err 'the key is invalid' in {}", FN_CTX),
        };

        Ok(payment)
    }

    fn set_payment(&self, pubkey: &PubkeyKey, payment: &Payment) -> anyhow::Result<()> {
        const FN_CTX: &str = "set_payment()";

        let b = sonic_rs::to_vec(payment)
            .context(concatcp!("err sonic_rs::to_vec() in ", FN_CTX))?;

        let options = WriteOptions::new();
        self.db.put(options, pubkey, &b)
            .context(concatcp!("err db.put() in ", FN_CTX))?;

        Ok(())
    }

    fn remove_payment(&self, pubkey: &PubkeyKey) -> anyhow::Result<()>{
        const FN_CTX: &str = "remove_payment()";

        let options = WriteOptions::new();
        self.db.delete(options, pubkey)
            .context(concatcp!("err db.delete() in ", FN_CTX))?;

        Ok(())
    }
}