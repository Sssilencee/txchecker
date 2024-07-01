use sonic_rs::Deserialize;

pub type DeliveryTag = u64;

#[derive(Deserialize, Debug)]
pub struct TransferMsg {
    pub id: String,
    pub address: String,
    pub amount: u64,
}

pub struct ConsumerMsg<T> {
    pub msg: T,
    pub tag: DeliveryTag,
}

impl<T> ConsumerMsg<T> {
    pub fn new(msg: T, tag: DeliveryTag) -> Self {
        Self { msg, tag }
    }
}