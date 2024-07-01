use db_key::Key;
use leveldblib::slice_to_arr;
use log::error;

const HEIGHT_KEY: u8 = 1;

pub struct HeightKey(u8);

impl Key for HeightKey {
    fn as_slice<T, F: Fn(&[u8]) -> T>(&self, f: F) -> T {
        f(&self.0.to_le_bytes())
    }

    fn from_u8(key: &[u8]) -> Self {
        if key.len() != 1 {
            error!("err not u8 key used as database key in from_u8()");
            return Default::default();
        }

        HeightKey(u8::from_le_bytes(slice_to_arr(key)))
    }
}

impl Default for HeightKey {
    fn default() -> Self {
        Self(HEIGHT_KEY)
    }
}