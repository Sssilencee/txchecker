use tokio::sync::mpsc::UnboundedSender;

pub type Slot = u64;
pub type SlotTx = UnboundedSender<Slot>;

pub const SLOT_CONFIRMATION_LAG: Slot = 40;