use anyhow::{bail, Context};
use app::domain::height::Height;
use const_format::concatcp;
use leveldb::{database::Database, kv::KV, options::{ReadOptions, WriteOptions}};
use leveldblib::slice_to_arr;
use tokio::{select, sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender}, task};
use tokio_util::sync::CancellationToken;

use super::models::HeightKey;

pub struct HeightActor {
    height_rx: UnboundedReceiver<Height>,
    db: Database<HeightKey>,
}

impl HeightActor {
    pub fn new(db: Database<HeightKey>) -> (Self, UnboundedSender<Height>) {
        let (tx, height_rx) = unbounded_channel();
        (Self { height_rx, db }, tx)
    }

    pub async fn start(mut self, token: CancellationToken) -> anyhow::Result<()> {
        loop {
            select! {
                Some(height) = self.height_rx.recv() => self.set_height(height)?,

                _ = token.cancelled() => return Ok(()),

                _ = task::yield_now() => continue,
            }
        }
    }

    fn set_height(&self, height: Height) -> anyhow::Result<()> {
        let options = WriteOptions::new();
        self.db.put(options, HeightKey::default(), &height.to_le_bytes())
            .context("err db.put() in set_height()")?;

        Ok(())
    }

    pub fn get_height(&self) -> anyhow::Result<Option<Height>> {
        const FN_CTX: &str = "get_height()";

        let options = ReadOptions::new();
        let b = self.db.get_bytes(options, HeightKey::default())
            .context(concatcp!("err db.get() in ", FN_CTX))?;

        let height = match b {
            Some(b) => match b.len() {
                8 => Some(Height::from_le_bytes(slice_to_arr(&b))),
                _ => bail!("err not u64 number used as database entry in {}", FN_CTX),
            },
            None => None,
        };

        Ok(height)
    }
}