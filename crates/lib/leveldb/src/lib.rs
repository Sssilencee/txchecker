use std::path::Path;

use db_key::Key;
use leveldb::{database::Database, options::Options};

pub fn connect<T: Key>(path: &str) -> anyhow::Result<Database<T>>{
    let path = Path::new(path);

    let mut options = Options::new();
    options.create_if_missing = true;

    Ok(Database::open(path, options)?)
}

pub fn slice_to_arr<const N: usize>(slice: &[u8]) -> [u8; N] {
    let mut arr = [0; N];
    arr.copy_from_slice(slice);
    arr
}