use serde::{Deserialize, Serialize};

use crate::consumer::messages::DeliveryTag;

#[derive(Serialize, Deserialize, Debug)]
pub struct ResultMsg {
    pub id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub signatures: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<()>
}

impl ResultMsg {
    pub fn new(
        id: String,
        signatures: Option<Vec<String>>,
        error: Option<()>,
    ) -> Self {
        Self { id, signatures, error }
    }
}

#[derive(Debug)]
pub struct ProducerMsg<T> {
    pub msg: T,
    pub tag: DeliveryTag,
}

impl<T> ProducerMsg<T> {
    pub fn new(msg: T, tag: DeliveryTag) -> Self {
        Self { msg, tag }
    }
}