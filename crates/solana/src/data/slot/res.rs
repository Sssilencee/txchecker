use sonic_rs::Deserialize;

use crate::domain::slot::Slot;

pub type SubscriptionId = u64;

#[derive(Deserialize)]
pub struct SlotNotification {
    pub slot: Slot,
}