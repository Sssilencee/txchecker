use anyhow::Context;
use app::{application::transfer::ParserPort, domain::pubkey::{Pubkey, ED25519_PUBKEY_LEN}};

pub struct Parser;

impl ParserPort for Parser {
    fn to_pubkey(&self, address: &String) -> anyhow::Result<Pubkey> {
        to_pubkey(address)
    }
}

pub fn to_pubkey(address: &String) -> anyhow::Result<Pubkey> {
    let mut pk = [0; ED25519_PUBKEY_LEN];

    bs58::decode(address)
        .onto(&mut pk)
        .context("err bs58::decode() in to_pubkey()")?;

    Ok(Pubkey::Ed25519(pk))
}