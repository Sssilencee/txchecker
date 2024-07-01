use std::sync::Arc;

use anyhow::{bail, Result};
use http_body_util::Empty;
use hyper::{body::Bytes, header::{CONNECTION, UPGRADE}, upgrade::Upgraded, Request};
use hyper_util::rt::{TokioExecutor, TokioIo};
use fastwebsockets::{handshake, FragmentCollector};
use rustls::{ClientConfig, RootCertStore};
use tokio::net::TcpStream;
use tokio_rustls::{TlsConnector, client::TlsStream};
use url::Url;
use webpki_roots::TLS_SERVER_ROOTS;

pub async fn connect(url: &str) -> Result<FragmentCollector<TokioIo<Upgraded>>> {
    let url_parsed = Url::parse(url)?;
    let host = url_parsed.host_str().unwrap_or_default();
    let port = url_parsed
        .port_or_known_default()
        .unwrap_or_default();
    let addr = format!("{host}:{port}");

    let req = Request::builder()
        .method("GET")
        .uri(url)
        .header("Host", host)
        .header(UPGRADE, "websocket")
        .header(CONNECTION, "upgrade")
        .header(
        "Sec-WebSocket-Key",
        fastwebsockets::handshake::generate_key(),
        )
        .header("Sec-WebSocket-Version", "13")
        .body(Empty::<Bytes>::new())?;

    let ex = TokioExecutor::new();
    let (ws, _) = match url_parsed.scheme() {
        "http" => {
            let stream = TcpStream::connect(addr).await?;
            handshake::client(&ex, req, stream).await?
        },

        "https" => {
            let stream = get_tls_stream(&addr, host).await?;
            handshake::client(&ex, req, stream).await?
        },

        scheme => bail!("err match url_parsed.scheme() in connect() unsupported scheme: {}", scheme),
    };

    Ok(FragmentCollector::new(ws))
}

async fn get_tls_stream(addr: &str, host: &str) -> Result<TlsStream<TcpStream>> {
    let stream = TcpStream::connect(addr).await?;
    let domain = host
        .to_owned()
        .try_into()?;

    let mut root_store = RootCertStore::empty();
    root_store.extend(TLS_SERVER_ROOTS.iter().cloned());

    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));

    Ok(connector.connect(domain, stream).await?)
}