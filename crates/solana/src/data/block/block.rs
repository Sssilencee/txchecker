use anyhow::Context;
use app::domain::transfer::IncomingTransfer;
use const_format::concatcp;
use http_body_util::{BodyExt, Full};
use hyper::{body::Bytes, Method, Request};
use hyper_util::client::legacy::{connect::HttpConnector, Client};

use crate::{data::{req::RpcReq, res::RpcRes}, domain::slot::Slot};

use super::res::GetBlockRes;

#[trait_variant::make(BlockRepo: Send)]
pub trait LocalBlockRepo {
    async fn get_block(&self, slot: Slot) -> anyhow::Result<Vec<IncomingTransfer>>;
}

pub struct BlockService {
    client: Client<HttpConnector, Full<Bytes>>,
    endpoint_url: String,
}

impl BlockService {
    pub fn new(client: Client<HttpConnector, Full<Bytes>>, endpoint_url: String) -> Self {
        Self { client, endpoint_url }
    }
}

impl BlockRepo for BlockService {
    async fn get_block(&self, slot: Slot) -> anyhow::Result<Vec<IncomingTransfer>> {
        const FN_CTX: &str = "get_block()";

        let req = RpcReq::new_get_block(slot);
        let payload = sonic_rs::to_vec(&req)
            .context(concatcp!("err sonic_rs::to_vec() in ", FN_CTX))?;

        let req = Request::builder()
            .uri(&self.endpoint_url)
            .method(Method::POST)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(payload)))
            .context(concatcp!("err Request::builder() in ", FN_CTX))?;

        let res = self.client.request(req)
            .await
            .context(concatcp!("err client.request() in ", FN_CTX))?;

        let b = res
            .collect()
            .await
            .context(concatcp!("err res.collect() in ", FN_CTX))?
            .to_bytes();

        let block: RpcRes<GetBlockRes> = sonic_rs::from_slice(&b)
            .context(concatcp!("err sonic_rs::from_slice() in ", FN_CTX))?;

        Ok(block.result.into())
    }
}