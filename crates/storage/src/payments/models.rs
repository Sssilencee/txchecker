use app::domain::pubkey::{Pubkey, ED25519_PUBKEY_LEN, SECP256K1_PUBKEY_LEN};
use db_key::Key;
use hashbrown::HashMap;
use leveldblib::slice_to_arr;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Hash, PartialEq, Eq)]
pub enum PubkeyKey {
    Ed25519([u8; ED25519_PUBKEY_LEN]),
    Secp256k1([u8; SECP256K1_PUBKEY_LEN]),
    Unknown,
}

impl Key for PubkeyKey {
    fn as_slice<T, F: Fn(&[u8]) -> T>(&self, f: F) -> T {
        f(self.as_ref())
    }

    fn from_u8(key: &[u8]) -> Self {
        use PubkeyKey::*;

        match key.len() {
            ED25519_PUBKEY_LEN => Ed25519(slice_to_arr(key)),
            SECP256K1_PUBKEY_LEN => Secp256k1(slice_to_arr(key)),

            _ => {
                error!("err invalid key length in Pubkey::from_u8()");
                Unknown
            }
        }
    }
}

impl AsRef<[u8]> for PubkeyKey {
    fn as_ref(&self) -> &[u8] {
        use PubkeyKey::*;

        match self {
            Ed25519(s) => s.as_slice(),
            Secp256k1(s) => s.as_slice(),

            Unknown => {
                error!("err unknown Pubkey used as a database key in Pubkey::as_slice()");
                &[]
            }
        }
    }
}

impl Into<PubkeyKey> for Pubkey {
    fn into(self) -> PubkeyKey {
        use Pubkey::*;

        match self {
            Ed25519(p) => PubkeyKey::Ed25519(p),
            Secp256k1(p) => PubkeyKey::Secp256k1(p),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Payment {
    pub id: String,
    pub tag: u64,
    pub amount: u64,
    pub signatures: Vec<String>,
}

impl Payment {
    pub fn new(id: String, tag: u64, amount: u64) -> Self {
        Self { id, tag, amount, signatures: Default::default() }
    }
}

pub type PaymentsCache = HashMap<PubkeyKey, Payment>;