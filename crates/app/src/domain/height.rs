use tokio::sync::mpsc::UnboundedSender;

pub type Height = u64;
pub type HeightTx = UnboundedSender<Height>;