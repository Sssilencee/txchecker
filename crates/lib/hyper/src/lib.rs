use http_body_util::Full;
use hyper::body::Bytes;
use hyper_util::{client::legacy::{connect::HttpConnector, Client}, rt::TokioExecutor};

pub fn connect() -> Client<HttpConnector, Full<Bytes>> {
    Client::builder(TokioExecutor::new())
        .http2_only(true)
        .build_http()
}