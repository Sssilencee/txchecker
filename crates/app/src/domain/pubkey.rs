pub const ED25519_PUBKEY_LEN: usize = 32;
pub const SECP256K1_PUBKEY_LEN: usize = 33;

#[derive(Debug)]
pub enum Pubkey {
    Ed25519([u8; ED25519_PUBKEY_LEN]),
    Secp256k1([u8; SECP256K1_PUBKEY_LEN]),
}