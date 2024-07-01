use sonic_rs::Serialize;

use crate::domain::slot::Slot;

use super::slot::res::SubscriptionId;

#[derive(Serialize)]
pub struct RpcReq<'a, T> {
    jsonrpc: &'a str,
    id: &'a str,
    method: &'a str,

    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<T>,
}

impl<'a, T> RpcReq<'a, T> {
    pub fn new(method: &'a str, params: Option<T>) -> Self {
        Self { jsonrpc: "2.0", id: "1", method, params }
    }

    pub fn new_slot_subscribe() -> Self {
        Self::new("slotSubscribe", None)
    }
}

impl<'a> RpcReq<'a, [RpcParameter; 2]> {
    pub fn new_get_block(slot: Slot) -> Self {
        let config = RpcConfig::builder()
            .with_commitment(Commitment::Finalized)
            .with_encoding(Encoding::Json);

        Self::new("getBlock", Some([
            RpcParameter::Slot(slot),
            RpcParameter::RpcConfig(config),
        ]))
    }
}

impl<'a> RpcReq<'a, [RpcParameter; 1]> {
    pub fn new_slot_unsubscribe(subscription_id: SubscriptionId) -> Self {
        Self::new("slotUnsubscribe", Some([
            RpcParameter::SubscriptionId(subscription_id),
        ]))
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum RpcParameter {
    RpcConfig(RpcConfig),
    Slot(Slot),
    SubscriptionId(SubscriptionId),
}

#[derive(Serialize)]
pub struct RpcConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    commitment: Option<Commitment>,

    #[serde(skip_serializing_if = "Option::is_none")]
    encoding: Option<Encoding>,
}

impl RpcConfig {
    fn builder() -> Self {
        Self { commitment: None, encoding: None }
    }

    fn with_commitment(mut self, commitment: Commitment) -> Self {
        self.commitment = Some(commitment);
        self
    }

    fn with_encoding(mut self, encoding: Encoding) -> Self {
        self.encoding = Some(encoding);
        self
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
enum Commitment {
    Finalized,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
enum Encoding {
    Json,
}